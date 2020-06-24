<?php

/**
 *
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
 * Module prefilter.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 */

class Text_Wiki_Parse_Modulepre extends Text_Wiki_Parse {

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
                        '^' .                     # Require start of line
                        '\[\[module654' .         # Start opening tag
                        '\s' . 
                        '([a-z0-9_\-\/]+)' .      # Module name, alphanumeric + some chars
                        '(\s+.*?)?' .             # Optional module parameters
                        '\]\]' .                  # End opening tag
                        '\s*\n' . 
                        '(?:' . 
                            '(.*?)' .             # Content betweent tags - no nesting
                            '\[\[\/module\]\]' .  # Closing tag
                        ')?' .                    # The content and end tag is optional
                        '/ismx';

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

        return preg_replace('/^\[\[module654/', '[[module', $matches[0]);
        # Seems to just replace module 654 with standard.
        # Backwards compatibility maybe?

    }
}
