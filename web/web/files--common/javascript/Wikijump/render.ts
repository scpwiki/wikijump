import { compress } from "compress-tag";

const AVATARS = "/common--images/avatars";
const KARMA = "/user--karma";

export const render = {
  printuser: function (
    userId: number,
    userName: string,
    showAvatar: boolean
  ): string {
    /**
     * Generates a HTML string containing a link to a user.
     *
     * @param userId: The ID of the user.
     * @param userName: The current name of the user.
     * @param showAvatar: Whether to the show the user's avatar and karma
     * indicator.
     */
    const link = compress`
      href="javascript:;"
      onclick="Wikijump.page.listeners.userInfo(${userId})"
    `;
    return compress`
      <span class="printuser">
      ${showAvatar
        ? `<a ${link}>
          <img class="small"
               src="${AVATARS}/${Math.floor(userId / 1000)}/${userId}/a16.png"
               alt=""
               style="background-image:url(${KARMA}/${userId})"/></a>`
        : ""
      }
      <a ${link}>${userName}</a></span>
    `;
  }
};
