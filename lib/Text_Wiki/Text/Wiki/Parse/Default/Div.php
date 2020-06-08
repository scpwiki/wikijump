<?php

/**
 *
 * Parses divs.
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
 * Parses divs.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 */

class Text_Wiki_Parse_Div extends Text_Wiki_Parse {

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
    public $regex = '/(\n)?\[\[div(\s.*?)?\]\] *\n((?:(?R)|.)*?)\[\[\/div\]\] */msi';

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
        $args = array();
        if ($attr['class']) {
            $args['class'] = $attr['class'];
        }
        if ($attr['style']) {
            $style = $attr['style'];
            $style = preg_replace('/visibility:\s*hidden;?/is', '', $style);
            $style = preg_replace('/display:\s*none;?/is', '', $style);
            $args['style'] = $style;
        }
        $options = array('args' => $args, 'type' => 'start');

        $start = $this->wiki->addToken($this->rule, $options);

        $end = $this->wiki->addToken($this->rule, array(
            'type' => 'end'));

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

}
