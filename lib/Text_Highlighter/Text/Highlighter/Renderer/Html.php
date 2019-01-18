<?php
/* vim: set expandtab tabstop=4 shiftwidth=4 softtabstop=4: */
/**
 * HTML renderer
 *
 * PHP versions 4 and 5
 *
 * LICENSE: This source file is subject to version 3.0 of the PHP license
 * that is available through the world-wide-web at the following URI:
 * http://www.php.net/license/3_0.txt.  If you did not receive a copy of
 * the PHP License and are unable to obtain it through the web, please
 * send a note to license@php.net so we can mail you a copy immediately.
 *
 * @category   Text
 * @package    Text_Highlighter
 * @author     Andrey Demenev <demenev@gmail.com>
 * @copyright  2004-2006 Andrey Demenev
 * @license    http://www.php.net/license/3_0.txt  PHP License
 * @version    CVS: $Id: Html.php,v 1.1 2006/08/03 13:41:00 redbeard Exp $
 * @link       http://pear.php.net/package/Text_Highlighter
 */

/**
 * @ignore
 */

require_once 'Text/Highlighter/Renderer.php';

// BC trick : only define constants if Text/Highlighter.php
// is not yet included
if (!defined('HL_NUMBERS_LI')) {
    /**#@+
     * Constant for use with $options['numbers']
     * @see Text_Highlighter_Renderer_Html::_init()
     */
    /**
     * use numbered list
     */
    define ('HL_NUMBERS_LI'    ,    1);
    /**
     * Use 2-column table with line numbers in left column and code in  right column.
     */
    define ('HL_NUMBERS_TABLE'    , 2);
    /**#@-*/
}

/**
 * HTML renderer
 *
 * Elements of $options argument of constructor (each being optional):
 *
 * - 'numbers' - Line numbering style 0 or {@link HL_NUMBERS_LI} or {@link HL_NUMBERS_TABLE}
 * - 'tabsize' - Tab size
 *
 * @author Andrey Demenev <demenev@gmail.com>
 * @category   Text
 * @package    Text_Highlighter
 * @copyright  2004-2006 Andrey Demenev
 * @license    http://www.php.net/license/3_0.txt  PHP License
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Highlighter
 */

class Text_Highlighter_Renderer_Html extends Text_Highlighter_Renderer
{

    /**#@+
     * @access private
     */

    /**
     * CSS class of last outputted text chunk
     *
     * @var string
     */
    public $_lastClass;

    /**
     * Line numbering style
     *
     * @var integer
     */
    public $_numbers = 0;

    /**
     * Tab size
     *
     * @var integer
     */
    public $_tabsize = 4;

    /**
     * Highlighted code
     *
     * @var string
     */
    public $_output = '';

    /**#@-*/


    /**
     * Preprocesses code
     *
     * @access public
     *
     * @param  string $str Code to preprocess
     * @return string Preprocessed code
     */
    function preprocess($str)
    {
        // normalize whitespace and tabs
        $str = str_replace("\r\n","\n", $str);
        // some browsers refuse to display mepty lines
        $str = preg_replace('~^$~m'," ", $str);
        $str = str_replace("\t",str_repeat(' ', $this->_tabsize), $str);
        return rtrim($str);
    }


    /**
     * Resets renderer state
     *
     * @access protected
     *
     *
     * Descendents of Text_Highlighter call this method from the constructor,
     * passing $options they get as parameter.
     */
    function reset()
    {
        $this->_output = '';
        $this->_lastClass = '';
        if (isset($this->_options['numbers'])) {
            $this->_numbers = (int)$this->_options['numbers'];
            if ($this->_numbers != HL_NUMBERS_LI && $this->_numbers != HL_NUMBERS_TABLE) {
                $this->_numbers = 0;
            }
        }
        if (isset($this->_options['tabsize'])) {
            $this->_tabsize = $this->_options['tabsize'];
        }
    }



    /**
     * Accepts next token
     *
     * @access public
     *
     * @param  string $class   Token class
     * @param  string $content Token content
     */
    function acceptToken($class, $content)
    {
        $theClass = $this->_getFullClassName($class);
        $content = htmlspecialchars($content);
        if (!$this->_output || $class != $this->_lastClass) {
            $tag = '';
            if ($this->_output) {
                $tag .= '</span>';
            }
            $tag .= '<span class="hl-' . $theClass . '">';
            $this->_output .= $tag;
        } else {
            $class = $this->_lastClass;
            $theClass = $this->_getFullClassName($class);
        }

        // make coloring tags not cross the list item tags
        if ($this->_numbers == HL_NUMBERS_LI) {
            $tag = "</span>\n<span class=\"hl-" . $theClass . '">';
            $content = str_replace("  ", '&nbsp; ', $content);
            $content = str_replace("\n", $tag, $content);
        }

        $this->_output .= $content;
        $this->_lastClass = $class;
    }

    /**
     * Given a CSS class name, returns the class name
     * with language name prepended, if necessary
     *
     * @access private
     *
     * @param  string $class   Token class
     */
    function _getFullClassName($class)
    {
        if (!empty($this->_options['use_language'])) {
            $theClass = $this->_language . '-' . $class;
        } else {
            $theClass = $class;
        }
        return $theClass;
    }

    /**
     * Signals that no more tokens are available
     *
     * @access public
     *
     */
    function finalize()
    {
        if ($this->_output) {
            $this->_output .= '</span>';
        }
        if ($this->_numbers == HL_NUMBERS_LI) {
            /* additional whitespace for browsers that do not display
            empty list items correctly */
            $this->_output = '<li>&nbsp;' . str_replace("\n", "</li>\n<li>&nbsp;", $this->_output) . '</li>';
            $this->_output = '<ol class="hl-main">' . $this->_output . '</ol>';
        }
        if ($this->_numbers == HL_NUMBERS_TABLE) {
            $numbers = '';
            $nlines = substr_count($this->_output,"\n")+1;
            for ($i=1; $i<=$nlines; $i++) {
                $numbers .= $i . "\n";
            }
            $this->_output = '<table class="hl-table" width="100%"><tr>' .
                             '<td class="hl-gutter" align="right" valign="top" style="width:4ex;">' .
                             '<pre>' . $numbers . '</pre></td><td class="hl-main" valign="top"><pre>' .
                             $this->_output . '</pre></td></tr></table>';
        }
        if (!$this->_numbers) {
            $this->_output = '<div class="hl-main"><pre>' . $this->_output . '</pre></div>';
        }
    }

    /**
     * Get generated output
     *
     * @return string Highlighted code
     * @access public
     *
     */
    function getOutput()
    {
        return $this->_output;
    }
}

/*
 * Local variables:
 * tab-width: 4
 * c-basic-offset: 4
 * c-hanging-comment-ender-p: nil
 * End:
 */

?>
