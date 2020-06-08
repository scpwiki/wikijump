<?php
/**
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    $Id$
 * @link       http://pear.php.net/package/Text_Wiki
 */

/**
 * Footnote item.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_Footnoteitem extends Text_Wiki_Render {

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

            $idPrefix = $this->wiki->getRenderConf('xhtml', 'footnote', 'id_prefix');
            $id = $options['id'];
            $out = '<div class="footnote-footer" id="footnote-' . $idPrefix . $id . '">';
            $out .= '<a href="javascript:;" ' . 'onclick="WIKIDOT.page.utils.scrollToReference(\'footnoteref-' . $idPrefix . $id . '\')">' . $id . '</a>. ';
            return $out;
        }
        if ($options['type'] == 'end') {
            $out = '</div>';
            return $out;
        }
    }
}
