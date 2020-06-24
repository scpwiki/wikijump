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
 * Parses alignmnent blocks.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 */

class Text_Wiki_Parse_Divalign extends Text_Wiki_Parse {

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
                        '^' .                    # Start of line
                        '\[\[(=|<|>|==)\]\]' .   # Opening tag with variations
                        '\n' . 
                        '((?:(?R)|.)*?)' .       # Contents of tag - nesting is allowed
                        '\[\[\/\\1\]\]' .        # Closing tag that matches opening tag
                        '$' .                    # End of line
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
        $align = $matches[1];
        $content = $matches[2];

        $options = array('type' => 'start');

        $aligns = array('=' => 'center', '<' => 'left',
            '>' => 'right', '==' => 'justify');

        $align = $aligns[$align];

        if (!$align) {
            return;
        }

        $options['align'] = $align;

        $start = $this->wiki->addToken($this->rule, $options);

        $end = $this->wiki->addToken($this->rule, array(
            'type' => 'end'));

        return $start . "\n\n" . $content . "\n\n" . $end;

    }

    function parse() {
        $oldSource = $this->wiki->source;
        $this->wiki->source = preg_replace_callback($this->regex, array(
            &$this, 'process'), $this->wiki->source);
        if ($oldSource != $this->wiki->source) {
            $this->parse();
        }
    }

}
