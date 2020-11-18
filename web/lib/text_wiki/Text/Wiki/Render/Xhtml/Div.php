<?php
/**
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    $Id$
 * @link       http://pear.php.net/package/Text_Wiki
 */

/**
 * Renders a div element.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_Div extends Text_Wiki_Render {

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
        if ($options['type'] == 'start') {
            $args = $options['args'];
            $argstr = '';
            foreach ($args as $key => $value) {
                $argstr .= ' ' . $key . '="' . $value . '"';
            }
            return "<div " . $argstr . ">";
        }

        if ($options['type'] == 'end') {
            return "</div>";
        }

    }
}
