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
 * Link to an uploaded file.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */

class Text_Wiki_Render_Xhtml_File extends Text_Wiki_Render {

    public $conf = array(
        'base' => '/'
    );

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
        $file = $options['file'];
        $anchor = $options['anchor'];

        if (strpos($file, '/') !== false) {
        		// ok, hardcode the path... sorry.
        		$file = preg_replace("/^\//", '', $file);
        		$file = "/local--files/".$file;
        }else{
        		$noLocal = $this->getConf("no_local");
        		if($noLocal){
        			return '<span class="error-inline">' .
        					'Sorry, local files without page name specified not allowed. ' .
        					'Use [[file <em>pagename</em>/<em>filename</em>]]</span>';
        		}
        		$file = $this->getConf('base', '/') . $file;
        }

        $output = "<a href=\"$file\">$anchor</a>";
        return $output;
    }
}
