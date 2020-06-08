<?php
// vim: set expandtab tabstop=4 shiftwidth=4 softtabstop=4:
/**
 * Toc rule end renderer for Xhtml
 *
 * PHP versions 4 and 5
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Paul M. Jones <pmjones@php.net>
 * @author Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    $Id$
 * @link       http://pear.php.net/package/Text_Wiki
 */

/**
 * This class inserts a table of content in XHTML.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Paul M. Jones <pmjones@php.net>
 * @author Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_Toc extends Text_Wiki_Render {

    public $conf = array(
        'css_list' => null,
        'css_item' => null,
        //'title' => '<strong>Table of Contents</strong>',
        'div_id' => 'toc',
        'collapse' => false
    );

    public $min = 0;

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

    function token($options)
    {
        // type, id, level, count, attr
        extract($options);

        switch ($type) {

        case 'list_start':

            $css = $this->getConf('css_list');
            $html = '';

            // add the div, class, and id
            if(!$options['align']){
            		// explorero stupido.
            		$html .= '<table style="margin:0; padding:0"><tr><td style="margin:0; padding:0">';
            }
            $html .= '<div';

			$class = null;
			if($options['align'] == "f<"){
				$class="floatleft";
			}elseif($options['align'] == "f>"){
				$class="floatright";
			}
            $div_id = $this->getConf('div_id');
            if ($div_id) {
                $html .= " id=\"$div_id\"";
            }
			if($class){
				$html .= ' class="'.$class.'" ';
			}
            // add the title, and done
            $html .= '>';
            $html .= '<div id="toc-action-bar"><a href="javascript:;" onclick="WIKIDOT.page.listeners.foldToc(event)">'._('fold').'</a><a style="display: none" href="javascript:;" onclick="WIKIDOT.page.listeners.unfoldToc(event)">'._('unfold').'</a></div>';
            $html .= '<div class="title">'._('Table of Contents').'</div>';
            $html .= '<div id="toc-list">';
            return $html;
            break;

        case 'list_end':
        		$out = '';
        		if(!$options['align']){
            		$out .= "</td></tr></table>";
            }
           	return "\n</div></div>$out\n\n";
            break;

        case 'item_start':
            $html = "\n\t<div";

            $css = $this->getConf('css_item');
            if ($css) {
                $html .= " class=\"$css\"";
            }

            $pad = ($level - $this->min);
            $html .= " style=\"margin-left: {$pad}em;\">";

			$d = utf8_encode("\xFC");
            $html .= "<a href=\"#$id\">$d$d";
            return $html;
            break;

        case 'item_end':
        	$d = utf8_encode("\xFC");
            return "$d$d</a></div>";
            break;
        }
    }
}
