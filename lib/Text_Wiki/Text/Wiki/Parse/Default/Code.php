<?php

/**
 *
 * Parses for text marked as a code example block.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Paul M. Jones <pmjones@php.net>
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
 * Parses for text marked as a code example block.
 *
 * This class implements a Text_Wiki_Parse to find sections marked as code
 * examples.  Blocks are marked as the string [[code]] on a line by itself,
 * followed by the inline code example, and terminated with the string
 * [[/code]] on a line by itself.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Paul M. Jones <pmjones@php.net>
 *
 * @author Michal Frackowiak
 *
 */

class Text_Wiki_Parse_Code extends Text_Wiki_Parse {

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

    public $regex = ';^\[\[code(\s[^\]]*)?\]\]((?:(?R)|.)*?)\[\[/code\]\](\s|$);msi';

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
        // are there additional attribute arguments?
        $args = trim($matches[1]);

        if ($args == '') {
            $options = array('text' => $matches[2],
                'attr' => array(
                    'type' => ''));
        } else {
            // get the attributes...
            $attr = $this->getAttrs($args);

            // ... and make sure we have a 'type'
            if (!isset($attr['type'])) {
                $attr['type'] = '';
            }

            // retain the options
            $options = array(
                'text' => $matches[2],
                'attr' => $attr);
        }

        return "\n\n" . $this->wiki->addToken($this->rule, $options) . $matches[3];
    }
}
