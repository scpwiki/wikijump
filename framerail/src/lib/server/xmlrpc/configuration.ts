export const xmlRpcAuthenticationConfiguration = () => ({
  ownerUsername: process.env.WIKIDOT_XMLRPC_OWNER_USERNAME?.trim() || undefined,
  writePassword: process.env.XML_RPC_WRITE_PASSWORD,
  writeUsername: process.env.XML_RPC_WRITE_USERNAME
})
