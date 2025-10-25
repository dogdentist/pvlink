import { exceptions } from "winston";
import { logger } from "./main";
import { openPgConn } from "./db";
import { Cookie, sql } from "bun";
import bcrypt from 'bcrypt';
import { openCacheConn } from "./cache";
import { parse as cookieParse } from "cookie";

type AuthLogin = {
  username: string,
  password: string
};

export type AuthCacheValue = {
  // username
  u: string
};

export const middlewareAuthCheck = async (remoteAddress: string, req: Bun.BunRequest, forward: (auth: AuthCacheValue) => Promise<Response>) => {
  const cookies = req.headers.get("cookie");

  if (!cookies) {
    logger.warn(`${remoteAddress} | missing auth cookie`);
    return new Response(null, { status: 401 });
  }

  const sessionToken = cookieParse(cookies).session;

  if (!sessionToken) {
    logger.warn(`${remoteAddress} | missing auth cookie`);
    return new Response(null, { status: 401 });
  }

  const redisConn = await openCacheConn();
  const authRaw = await redisConn.get(sessionToken);

  if (!authRaw) {
    logger.warn(`${remoteAddress} | unknown/invalid auth cookie`);
    return new Response(null, { status: 401 });
  }

  const auth: AuthCacheValue = JSON.parse(authRaw);

  return await forward(auth);
};

const createCookieKey = () => {
  const lut = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  const random = new Uint32Array(64);
  let result = '';

  crypto.getRandomValues(random);

  for (let i = 0; i < 64; i++) {
    result += lut[random[i]! % lut.length];
  }

  return result;
}

export const loginRoute = async (req: Bun.BunRequest, remoteAddress: string): Promise<Response> => {
  const data: AuthLogin = JSON.parse(await req.text());
  const pg = await openPgConn();

  const row = await pg<[string]>`
    SELECT tb_user_password FROM tb_user
    WHERE tb_user_username = ${data.username}
    LIMIT 1
  `.values();

  pg.close();

  if (row[0][0] != undefined) {
    if (await bcrypt.compare(data.password, row[0][0])) {
      const cookieKey = createCookieKey();
      const redisConn = await openCacheConn();

      await redisConn.set(cookieKey, JSON.stringify({
        u: data.username
      } as AuthCacheValue));

      redisConn.close();

      const cookie = new Cookie("session", cookieKey, {
        httpOnly: true,
        secure: true,
        sameSite: "strict",
        maxAge: 3600, // must the the same as the TTL of the Redis's container orchestration env
        path: "/",
      });

      logger.warn(`${remoteAddress} | user ${data.username} signed-in`);

      return new Response(null, {
        status: 200, headers: {
          "Set-Cookie": cookie.toString()
        }
      });
    } else {
      logger.warn(`${remoteAddress} | failed to login as ${data.username}`);

      return new Response(null, { status: 401 });
    }
  } else {
    logger.warn(`${remoteAddress} | attempted to login as non-existent user ${data.username}`);

    return new Response(null, { status: 401 });
  }
}
