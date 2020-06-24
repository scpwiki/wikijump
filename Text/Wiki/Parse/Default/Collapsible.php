<?php

/**
 *
 * Creates a collapsible content block.
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
 * Creates a collapsible content block.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 */

class Text_Wiki_Parse_Collapsible extends Text_Wiki_Parse {

    static $count = 0;

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
                        '(\n)?' .         # Start with an optional newline?
                        '\[\[' . 
                        'collapsible' .
                        '(\s.*?)?' .       # Parameters of collapsbile
                        '\]\]' . 
                        '(.*?)' .          # Contents of collapsible - no nesting
                        '\[\[\/collapsible\]\]\s*' . 
                        '/msix';

    /**
     *
     * Generates a token entry for the matched text.  Token options are:
     *
     * 'text' => The full matched text, not including the <code></code> tags.
     *
     * @access public
     *
     * @param array &$matches The array of matches from parse().
     *
     * @return A delimited token number to be used as a placeholder in
     * the source text.
     *
     */

    function process(&$matches) {

        $content = $matches[3];

        $attr = $this->getAttrs(trim($matches[2]));

        $options = array('args' => $attr, 'type' => 'start',
            'count' => self::$count);

        $start = $this->wiki->addToken($this->rule, $options);

        $end = $this->wiki->addToken($this->rule, array(
            'args' => $attr, 'type' => 'end',
            'count' => self::$count));
        self::$count++;
        return $matches[1] . $matches[1] . $start . "\n\n" . $content . "\n\n" . $end;

    }

}
