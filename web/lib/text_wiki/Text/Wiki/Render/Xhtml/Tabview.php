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
 * Tabs.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_Tabview extends Text_Wiki_Render {

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
            $out = <<<EOT
<script type="text/javascript" src="/common--javascript/yahooui/element-beta-min.js"></script>
<script type="text/javascript" src="/common--javascript/yahooui/tabview-min.js"></script>
EOT;
            $args = $options['args'];
            $out .= '<div id="wiki-tabview-' . $options['tabviewId'] . '" class="yui-navset"><ul class="yui-nav">';
            foreach ($options['tabs'] as $tab) {
                $class = '';
                if ($tab['tabId'] == 0) {
                    $class = ' class="selected"';
                }
                $out .= '<li' . $class . '><a href="javascript:;"><em>' . ($tab['title'] ? $tab['title'] : 'untitled') . '</em></a></li>';

            }
            $out .= '</ul>';
            $out .= '<div class="yui-content">';
            return $out;
        }

        if ($options['type'] == 'tabStart') {
            $style = '';
            if ($options['tabId'] != 0) {
                $style = ' style="display:none"';
            }
            return '<div id="wiki-tab-' . $options['tabviewId'] . '-' . $options['tabId'] . '"' . $style . '>';
        }
        if ($options['type'] == 'tabEnd') {
            return '</div>';
        }

        if ($options['type'] == 'end') {
            $out = '</div></div>';
            // load and execute the script! (bad hack)


            //$out .=  <<<EOT
            //<script type="text/javascript">
            //</script>


            $out .= <<<EOT
<script type="text/javascript">
(function(){
	var tabView{$options['tabviewId']} = new YAHOO.widget.TabView('wiki-tabview-{$options['tabviewId']}');
})();
</script>
EOT;

            return $out;
        }

    }
}
