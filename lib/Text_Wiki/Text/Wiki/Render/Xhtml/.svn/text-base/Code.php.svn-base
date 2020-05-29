<?php
require_once(WIKIDOT_ROOT."/lib/Text_Highlighter/Text/Highlighter.php");
ini_set('include_path',ini_get('include_path').':'.WIKIDOT_ROOT.'/lib/Text_Highlighter/');

/**
 * Code rule end renderer for Xhtml
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
 * This class renders code blocks in XHTML.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Paul M. Jones <pmjones@php.net>
 * @author Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_Code extends Text_Wiki_Render {

    public $conf = array(
        'css'      => null, // class for <pre>
        'css_code' => null, // class for generic <code>
        'css_php'  => null, // class for PHP <code>
        'css_html' => null, // class for HTML <code>
        'css_filename' => null // class for optional filename <div>
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
        $text = $options['text'];
        $attr = $options['attr'];
        $type = strtolower($attr['type']);

        $css      = $this->formatConf(' class="%s"', 'css');
        $css_code = $this->formatConf(' class="%s"', 'css_code');
        $css_php  = $this->formatConf(' class="%s"', 'css_php');
        $css_html = $this->formatConf(' class="%s"', 'css_html');
        $css_filename = $this->formatConf(' class="%s"', 'css_filename');

		$text = trim($text);
		
        if ($type == 'php') {
        	 	/*if (substr($text, 0, 5) != '<?php') {
                // PHP code example:
                // add the PHP tags
                $text = "<?php\n" . $text . "\n?>"; // <?php
          	}*/
        	 	$highlighter = Text_Highlighter::factory('php');
            $text = $highlighter->highlight($text);
        }elseif ($type == 'cpp') {
            $highlighter = Text_Highlighter::factory('cpp');
            $text = $highlighter->highlight($text);
        }elseif ($type == 'css') {
            $highlighter = Text_Highlighter::factory('css');
            $text = $highlighter->highlight($text);
        }elseif ($type == 'html' || $type == 'xhtml') {
            $highlighter = Text_Highlighter::factory('html');
            $text = $highlighter->highlight($text);
        }elseif ($type == 'diff') {
            $highlighter = Text_Highlighter::factory('diff');
            $text = $highlighter->highlight($text);
        }elseif ($type == 'dtd') {
            $highlighter = Text_Highlighter::factory('dtd');
            $text = $highlighter->highlight($text);
        }elseif ($type == 'java') {
            $highlighter = Text_Highlighter::factory('java');
            $text = $highlighter->highlight($text);
        }elseif ($type == 'javascript') {
            $highlighter = Text_Highlighter::factory('javascript');
            $text = $highlighter->highlight($text);
        }elseif ($type == 'sql') {
            $highlighter = Text_Highlighter::factory('sql');
            $text = $highlighter->highlight($text);
        }elseif ($type == 'perl') {
            $highlighter = Text_Highlighter::factory('perl');
            $text = $highlighter->highlight($text);
        }elseif ($type == 'python') {
            $highlighter = Text_Highlighter::factory('python');
            $text = $highlighter->highlight($text);
        }elseif ($type == 'ruby') {
            $highlighter = Text_Highlighter::factory('ruby');
            $text = $highlighter->highlight($text);
        }elseif ($type == 'sql') {
            $highlighter = Text_Highlighter::factory('sql');
            $text = $highlighter->highlight($text);
        }elseif($type=="xml"){
       	 	$highlighter = Text_Highlighter::factory('xml');
            $text = $highlighter->highlight($text);
        } else {
            // generic code example:
            // convert tabs to four spaces,
            // convert entities.
            $text = str_replace("\t", "    ", $text);
            $text = htmlspecialchars($text);
            $text = "<pre$css><code$css_code>$text</code></pre>";
        }
		
		 $text = "<div class=\"code\">".$text."</div>";
		
        if ($css_filename && isset($attr['filename'])) {
            $text = "<div$css_filename>" .
                $attr['filename'] . '</div>' . $text;
        }

        return "\n$text\n\n";
    }
}
