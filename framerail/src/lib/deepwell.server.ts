import axios from 'axios';

const DEEPWELL_HOST = process.env.DEEPWELL_HOST || 'localhost';
const DEEPWELL_PORT = 2747;

export async function ping(): string {
  // TODO for now, dump as string
  const response = await axios.put(
    `http://${DEEPWELL_HOST}:${DEEPWELL_PORT}/api/trusted/ping`,
     { timeout: 500 },
  );

  return response.data;
}
