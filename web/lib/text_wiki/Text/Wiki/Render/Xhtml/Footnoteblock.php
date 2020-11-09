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
 * Footnote block.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_Footnoteblock extends Text_Wiki_Render {

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
        $footnotes = $this->wiki->vars['footnotes'];
        if (count($footnotes) == 0) {
            return '';
        } // render nothing if no footnotes.
        if ($options['type'] == 'start') {
            $title = $options['title'];
            if ($title === null) {
                $title = _('Footnotes');
            }
            $out = '<div class="footnotes-footer">';
            if ($title !== "") {
                $out .= '<div class="title">' . htmlspecialchars($title) . '</div>';
            }
            return $out;
        }
        if ($options['type'] == 'end') {
            return '</div>';
        }
        /*
    		$footnotes = $this->wiki->vars['footnotes'];
    		if(count($footnotes) == 0){return '';} // render nothing if no footnotes.
    		$out = '<div class="footnotes-footer">';
    		$out .= '<div class="title">Footnotes</div>';
    		foreach($footnotes as $id => $content){
    			$out .= '<div class="footnote-footer" id="footnote-'.$id.'">';
    			$out .= '<a href="javascript:;" ' .
    					'onclick="WIKIDOT.page.utils.scrollToReference(\'footnoteref-'.$id.'\')">'.$id . '</a>. ' .
    							'<span class="content">'.$content.'</span>';
    			$out .= '</div>';
    		}
    		$out .= '</div>';
        return $out;
        */
    }
}
