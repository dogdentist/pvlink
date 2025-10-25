import { SQL } from "bun";

export const openPgConn = async () => {
  return new SQL(`postgres://${process.env.PVLINK_DB_USER}:${process.env.PVLINK_DB_PASSWORD}@${process.env.PVLINK_DB_FQDN}/${process.env.PVLINK_DB_NAME}`)
}
