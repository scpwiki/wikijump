<?php
// vim: set expandtab tabstop=4 shiftwidth=4 softtabstop=4:
/**
 * Horiz rule end renderer for Xhtml
 *
 * PHP versions 4 and 5
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    $Id$
 * @link       http://pear.php.net/package/Text_Wiki
 */

use Ozone\Framework\Database\Criteria;

use Wikidot\Utils\WDRenderUtils;
use Wikidot\Utils\WDStringUtils;
use Wikijump\Models\User;

/**
 * This class renders a user info.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_User extends Text_Wiki_Render {

    public $conf = array();

    /**
     *
     * Renders a token into text matching the requested format.
     *
     * @access public
     *
     * @param array $options The "options" portion of the token (second
     * element).
     *
     * @return string The text rendered from the token options.
     *
     */

    function token($options) {
        $userName = $options['userName'];
        $slug = WDStringUtils::toUnixName($userName);
        $user = User::firstWhere('slug', $slug);

        if ($user == null) {
            return '<span class="error-inline">' . sprintf(_('User <em>%s</em> cannot be found.'), $userName) . '</span>';
        } else {
            $o = array();
            if ($options['image']) {
                $o['image'] = true;
            }
            return WDRenderUtils::renderUser($user, $o);
        }

    }
}
