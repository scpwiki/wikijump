<?php

/**
 *
 * Parses for bib citations.
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
 * Parses for bibliography citations, e.g. ((bibcite somelabel)).
 *
 * This class implements a Text_Wiki_Parse to find math invocation code,
 * i.e. [[footnote]] text [[/footnote]]
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 */
class Text_Wiki_Parse_Bibcite extends Text_Wiki_Parse {

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
                        '\(\(' .             # Opening parens
                        'bibcite\s' .        # Module name and whitespace
                        '([a-z0-9]+)' .      # Alphanumeric citation
                        '\)\)' .             # Closing parens
                        '/ix';

    function process(&$matches) {
        $label = $matches[1];
        $options = array(
            'label' => $label);
        return $this->wiki->addToken($this->rule, $options);
    }
}
