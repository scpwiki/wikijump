<?php

/**
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 * @license LGPL
 *
 * @version $Id$
 *
 */

/**
 *
 * Parses for tabviews.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 */

class Text_Wiki_Parse_Tabview extends Text_Wiki_Parse {

    private static $_counter = 0;

    /**
     *
     * The regular expression used to find source text matching this
     * rule.
     *
     * @access public
     *
     * @var string
     *
     */

    public $regex =     '/' . 
                        '^' . 
                        '\[\[(?:tabview|tabs)(\s.*?)?\]\]' .   # Start tabview with parameters
                        '\s*' . 
                        '(' .                                  # Capture all tabs as a single group
                            '(?:\[\[tab(\s.*?)?\]\]' .         # Tab opening tag, with parameters
                            '.*?' .                            # Contents of tab - no nesting
                            '\[\[\/tab\]\]' .                  # Tab closing tag
                            '\s*' . 
                            ')+' .                             # Require at least one tab
                        ')' . 
                        '\[\[\/(?:tabview|tabs)\]\]\s*' .      # Tabview closing tag
                        '/msix';

    private $_startTabToken;
    private $_endTabToken;
    private $_tabs = array();

    private $_tabCounter = 0;

    function process(&$matches) {
        $this->_tabs = array();
        $this->_tabCounter = 0;

        $elId = self::$_counter;

        $content = $matches[2];

        $attr = $this->getAttrs(trim($matches[1]));

        $args = array();
        if ($attr['class']) {
            $args['class'] = $attr['class'];
        }

        // divide tabs - look for [[tab]]]...[[/tab]]


        $this->_startTabToken = $this->wiki->addToken($this->rule, array(
            'type' => 'tabStart',
            'tabviewId' => $elId));
        $this->_endTabToken = $this->wiki->addToken($this->rule, array(
            'type' => 'tabEnd', 'tabviewId' => $elId));

        // find tabs


        $content = preg_replace_callback(   '/' . 
                                            '\[\[tab' .            # Single tab opening tag
                                            '(\s+[^\]]+?)?' .      # Title of tab
                                            '(' .                  # Extract parameters
                                                '(?:\s+' . 
                                                '[a-z0-9\-_]+' . 
                                                '=' . 
                                                '"[^"]+"' .        # Parameter value is in quotes
                                                ')+' .             # At least one parameter, if any are present
                                            ')?' .                 # Parameters are optional
                                            '\]\]' . 
                                            '(.*?)' .              # Contents of tab - cannot contain [[tab]]
                                            '\[\[\/tab\]\]\n*' .   # Tab closing tag
                                            '/msix',
            array($this, '_handleTab'), $content);

        $options = array('args' => $args, 'type' => 'start',
            'tabs' => $this->_tabs,
            'tabviewId' => $elId);

        $start = $this->wiki->addToken($this->rule, $options);

        $end = $this->wiki->addToken($this->rule, array(
            'type' => 'end', 'tabviewId' => $elId));

        self::$_counter++;

        return $matches[1] . $matches[1] . $start . "\n\n" . $content . "\n\n" . $end;

    }

    function parse() {

        $oldSource = $this->wiki->source;
        $this->wiki->source = preg_replace_callback($this->regex, array(
            &$this, 'process'), $this->wiki->source);
        if ($oldSource != $this->wiki->source) {
            $this->parse();
        }
    }

    protected function _handleTab($matches) {
        $argString = trim($matches[2]);
        // bad hack - I will forgget how it works in a few minutes
        $ff = false;
        if (preg_match( '/' . 
                        '^' .                # Start of text
                        '[a-z0-9\-_]+' .     # Parameter name
                        '=' . 
                        '"[^"]+"' .          # Parameter value in quotes
                        '$' .                # End of text
                        '/six',
            trim($matches[1]))) {
                $argString .= ' ' . $matches[1];
                $ff = true;
            }
        $args = $this->getAttrs($argString);
        if (!isset($args['title']) && !$ff) {
            $args['title'] = trim($matches[1]);
        }
        $this->_tabs[] = array_merge($args, array(
            'tabId' => $this->_tabCounter));
        $content = $matches[3];
        $startTabToken = $this->wiki->addToken($this->rule, array(
            'type' => 'tabStart',
            'tabviewId' => self::$_counter,
            'tabId' => $this->_tabCounter));
        $this->_tabCounter++;
        return $startTabToken . "\n\n" . $content . "\n\n" . $this->_endTabToken;
    }

}
