
import { RedisClient } from 'bun';

export const openCacheConn = async () => {
  return new RedisClient(`redis://${process.env.PVLINK_CACHE_FQDN}`);
}

