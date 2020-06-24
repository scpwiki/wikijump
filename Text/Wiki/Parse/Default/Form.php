<?php

/**
 *
 * Parses for image placement.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Paul M. Jones <pmjones@php.net>
 * @author Michal Frackowiak
 *
 * @license LGPL
 *
 * @version $Id$
 *
 */

/**
 *
 * Parses for image placement.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Paul M. Jones <pmjones@php.net>
 * @author Michal Frackowiak
 *
 */

class Text_Wiki_Parse_Form extends Text_Wiki_Parse {

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
                        '\[\[form\]\]\s*\n' . 
                        '(.*)\n' .               # Anything then a newline
                        '---\s*\n' .             # Three hyphens (???)
                        '(.*)\n' .               # Anything again
                        '\[\[\/form\]\]' . 
                        '/isx';

    /**
     *
     * Generates a token entry for the matched text.  Token options are:
     *
     * 'src' => The image source, typically a relative path name.
     *
     * 'opts' => Any macro options following the source.
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
        $formYaml = $matches[1];
        $dataYaml = $matches[2];

        if (substr($dataYaml, 0, 2) == '%%') {
            $dataYaml = '';
        }
        $form = Wikidot_Form::fromYaml($formYaml, $dataYaml);

        $output = $this->wiki->addToken($this->rule, array('begin' => 1));

        foreach ($form->fields as $name => $field) {
            $output .= $this->wiki->addToken($this->rule, array(
                'field' => $field,
            ));
        }

        $output .= $this->wiki->addToken($this->rule, array('end' => 1));

        return $output;
    }
}
