import winston from "winston";
import DailyRotateFile from "winston-daily-rotate-file";
import { loginRoute, middlewareAuthCheck, type AuthCacheValue } from "./auth";
import { createSchema, createYoga, type YogaInitialContext } from "graphql-yoga";
import { openPgConn } from "./db";

export const logger = winston.createLogger({
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.printf(
      ({ timestamp, level, message, ...metadata }) =>
        `${timestamp} ${level.toUpperCase()} :: ${message} ${Object.keys(metadata).length ? JSON.stringify(metadata) : ""
          }`.trim()
    )
  ),
  transports: [
    new winston.transports.Console({
      level: "debug",
    }),
    new DailyRotateFile({
      filename: "/var/log/pvlink/service-%DATE%.txt",
      datePattern: "YYYY-MM-DD",
      maxFiles: `${process.env.PVLINK_LOG_RETENTION}d`,
      zippedArchive: false,
    }),
  ],
});

logger.info("started");

const getRequestIp = (req: Bun.BunRequest): string => {
  const value = req.headers.get("x-real-ip");

  if (value) {
    return value;
  } else {
    return "N/A";
  }
};

// dictates how many links are per page
const API_LINKS_ENTRIES_PER_PAGE = 10;

interface Context extends YogaInitialContext {
  auth: AuthCacheValue;
  remoteAddress: string;
}

const yoga = createYoga<Context>({
  graphqlEndpoint: "/api",
  schema: createSchema({
    typeDefs: `
      type Query {
        fetchLinkPagesCount: LinkPageCount!
        fetchLinksPages(page: Int = 1): LinkPage!
        fetchLinkClicks(linkId: Int!): LinkClicks
        fetchLinkCountryClicks(linkId: Int!): [LinkCountryClicks]!
      }

      type Mutation {
        deleteLink(linkId: Int!): Boolean!
        createLink(shortLink: String!, targetLink: String!, expiration: Int): Boolean!
      }

      type LinkPageCount {
        totalPages: Int!
      }

      type LinkPage {
        links: [Link!]!
      }

      type Link {
        id: Int!
        shortLink: String!
        longLink: String!
        expires: Int
        created: Int!
        clicks: Int!
      }

      type LinkClicks {
        clicks: Int!
      }

      type LinkCountryClicks {
        country: String!
        clicks: Int!
      }
    `,
    resolvers: {
      Mutation: {
        deleteLink: async (_parent, args: {
          linkId: number
        }, context) => {
          const pg = await openPgConn();

          try {
            return await pg.begin(async sql => {
              const result = await sql<[number]>`
                SELECT tb_link_id FROM tb_link
                WHERE tb_link_id = ${args.linkId}
                LIMIT 1
              `;

              if (result.length > 0) {
                await sql`
                  DELETE FROM tb_link
                  WHERE tb_link_id = ${args.linkId}
                `;

                return true;
              } else {
                throw Error("link doeesn't exist");
              }
            });
          } catch (err) {
            if (Error.isError(err) && err.message == "link doeesn't exist") {
              return false;
            }

            logger.error(`${context.remoteAddress} | deleteLink failed, ${err}`);

            throw Error("bad request");
          } finally {
            pg.close();
          }
        },
        createLink: async (_parent, args: {
          shortLink: string,
          targetLink: string,
          expiration?: number
        }, context) => {
          const pg = await openPgConn();

          try {
            return await pg.begin(async sql => {
              const result = await sql<[number]>`
                SELECT tb_link_id FROM tb_link
                WHERE tb_link_short = ${args.shortLink}
                LIMIT 1
              `;

              if (!result.length) {
                if (args.expiration) {
                  await sql`
                    INSERT INTO tb_link (
                      tb_link_short,
                      tb_link_long,
                      tb_link_expires,
                    ) VALUES (
                      ${args.shortLink},
                      ${args.targetLink},
                      ${sql`to_timestamp(${args.expiration})`}
                    )
                  `;
                } else {
                  await sql`
                    INSERT INTO tb_link (
                      tb_link_short,
                      tb_link_long
                    ) VALUES (
                      ${args.shortLink},
                      ${args.targetLink}
                    )
                  `;
                }

                return true;
              } else {
                return false;
              }
            });
          } catch (err) {
            if (Error.isError(err) && err.message == "link doeesn't exist") {
              return false;
            }

            logger.error(`${context.remoteAddress} | createLink failed, ${err}`);

            throw Error("bad request");
          } finally {
            pg.close();
          }
        }
      },
      Query: {
        fetchLinksPages: async (_parent, args: {
          page?: number
        }, context) => {
          type OrmLink = {
            id: number,
            shortLink: string,
            longLink: string,
            created: number,
            expires: number,
            clicks: number
          };

          const pg = await openPgConn();

          try {
            const result = await pg<[OrmLink]>`
              SELECT 
                tb_link_id :: integer as "id",
                tb_link_short as "shortLink",
                tb_link_long as "longLink", 
                EXTRACT(EPOCH FROM tb_link_creation) :: integer as "created",
                EXTRACT(EPOCH FROM tb_link_expires) :: integer as "expires",
                tb_link_clicks :: integer as "clicks"
              FROM tb_link
              LIMIT ${API_LINKS_ENTRIES_PER_PAGE}
              OFFSET ${((args.page ?? 1) - 1) * API_LINKS_ENTRIES_PER_PAGE}
            `;

            return {
              links: result,
            };
          } catch (err) {
            logger.error(`${context.remoteAddress} | fetchLinksPages failed, ${err}`);

            throw new Error("bad request");
          } finally {
            pg.close();
          }
        },
        fetchLinkPagesCount: async (_parent, _args: {}, context) => {
          type OrmCount = {
            count: number,
          };

          const pg = await openPgConn();

          try {
            const result = await pg<[OrmCount]>`
              SELECT COUNT(*) as count FROM tb_link
            `;

            return {
              totalPages: Math.ceil((result[0]?.count ?? 0) / API_LINKS_ENTRIES_PER_PAGE) || 1
            };
          } catch (err) {
            logger.error(`${context.remoteAddress} | fetchLinkPagesCount failed, ${err}`);

            throw new Error("bad request");
          } finally {
            pg.close();
          }
        },
        fetchLinkClicks: async (_parent, args: {
          linkId: number
        }, context) => {
          type OrmLink = {
            clicks: number,
          };

          const pg = await openPgConn();

          try {
            const result = await pg<OrmLink[]>`
              SELECT 
                tb_link_clicks :: integer as "clicks"
              FROM tb_link
              WHERE tb_link_id = ${args.linkId}
              LIMIT 1
            `;

            if (result && result[0]) {
              return {
                clicks: result[0].clicks
              }
            } else {
              return null;
            }
          } catch (err) {
            logger.error(`${context.remoteAddress} | fetchLinkClicks failed, ${err}`);

            throw new Error("bad request");
          } finally {
            pg.close();
          }
        },
        fetchLinkCountryClicks: async (_parent, args: {
          linkId: number
        }, context) => {
          type OrmLink = {
            country: string,
            clicks: number,
          };

          const pg = await openPgConn();

          try {
            const result = await pg<[OrmLink]>`
              SELECT 
                tb_link_clicks_country as "country",
                tb_link_clicks_clicks :: integer as "clicks"
              FROM tb_link_clicks
              WHERE tb_link_clicks_link_id = ${args.linkId}
            `;

            return result;
          } catch (err) {
            logger.error(`${context.remoteAddress} | fetchLinkCountryClicks failed, ${err}`);

            throw new Error("bad request");
          } finally {
            pg.close();
          }
        }
      }
    }
  }),
});

Bun.serve({
  port: 80,
  hostname: "0.0.0.0",
  routes: {
    '/api/auth/login': async (req) => {
      if (req.method == "POST") {
        const remoteAddress = getRequestIp(req);

        try {
          return await loginRoute(req, remoteAddress);
        } catch (err) {
          // bad requests/internal error is just 401 - hide the errors
          logger.error(`${remoteAddress} | bad request on login, ${err}`);
          return new Response(null, { status: 401 });
        }
      } else {
        return new Response(null, { status: 404 });
      }
    },
    '/api': async (req) => {
      const remoteAddress = getRequestIp(req);

      try {
        return await middlewareAuthCheck(remoteAddress, req, async (auth: AuthCacheValue): Promise<Response> => {
          const context: Context = {
            request: req,
            auth,
            remoteAddress,
            params: {},
            waitUntil: async () => { }
          };

          const res = await yoga.handleRequest(req, context);

          logger.info(`${remoteAddress} | status: ${res.status}`);

          return res;
        });
      } catch (err) {
        // bad requests/internal error is just 401 - hide the errors
        logger.error(`request failed, ${err}`);
        return new Response(null, { status: 401 });
      }
    }
  },
  fetch(_req) {
    return new Response(null, { status: 404 });
  },
});
