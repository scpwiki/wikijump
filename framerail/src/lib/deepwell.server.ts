import axios from 'axios';

const DEEPWELL_HOST = 'localhost';  // is 'api' in local container
const DEEPWELL_PORT = 2747;

export async function ping(): string {
  // TODO for now, dump as JSON
  const response = await axios.get(
    `http://${DEEPWELL_HOST}:${DEEPWELL_PORT}/api/trusted/ping`,
     { timeout: 500 },
  );

  return response.data;
}
