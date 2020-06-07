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
 * Renders embedded code.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_Embed extends Text_Wiki_Render {

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

    public $patterns = array();

    public function Text_Wiki_Render_Xhtml_Embed($obj) {
        parent::Text_Wiki_Render($obj);

        $patternDir = WIKIDOT_ROOT . '/conf/wikiparser/embed';
        $files = glob($patternDir . '/*.php');
        foreach ($files as $f) {
            require $f;
            $this->_patterns = array_merge($this->patterns, $patterns);
        }

    }

    function token($options) {

        $content = trim($options['content']);

        foreach ($this->_patterns as $pattern) {
            if (preg_match($pattern, $content)) {
                $content = preg_replace('/language="JavaScript[^"]*"/i', '', $content);
                // check for js events
                $eventpattern = 'on[a-z]+';
                $eventpattern = ';<[^>]*\s+' . $eventpattern . '\s*=[^>]+>;si';
                if (preg_match($eventpattern, $content)) {
                    break;
                }
                return $content;
            }
        }

        //no match...
        return '<div class="error-block">Sorry, no match for the embedded content.</div>';

    }
}
