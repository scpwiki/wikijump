<?php

/**
 *
 * Parses the [[bibliography]] block.
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
 * Parses the [[bibliography]] block.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 */

class Text_Wiki_Parse_Bibliography extends Text_Wiki_Parse {

    public $regex = null;

    function parse() {
        $regex =    '/' . 
                    '^' .                         # Start of line
                    '\[\[bibliography' .          # Tag name
                    '(\s+[^\]]+)?' .              # Parameters
                    '\]\]' . 
                    '(.*?)' .                     # Contents
                    '\[\[\/bibliography\]\]' .    # End tag
                    '[\s]*$' .                    # Allow whitespace until end of the line
                    '/smx';
        $this->wiki->source = preg_replace_callback($regex,
            array(&$this, 'process'), $this->wiki->source, 1);
    }

    function process(&$matches) {
        $inner = $matches[2];
        $args = $this->getAttrs($matches[1]);
        $title = $args['title'];
        if ($this->wiki->vars['bibitems'] == null) {
            $this->wiki->vars['bibitems'] = array();
            $this->wiki->vars['bibitemIds'] = array();
        }
        // parse the "inner" manually inserting delimiters
        $bi = $this->wiki->parseObj['Bibitem'];

        $inside = preg_replace_callback(
            '/' . 
            '^' .                # Start of line
            ':\s?' .             # Colon, then optional whitespace
            '([a-z0-9]+)' .      # Lowercase alphanumeric bib item name
            '\s?' .              # Optional whitespace
            ':\s' .              # Colon then mandatory whitespace
            '(.*)' .             # Rest of the line is the item definition
            '$' .                # End of line
            '/mix',
            array(&$bi, 'process'), $inner);

        return "\n" . $this->wiki->addToken($this->rule, array(
            'type' => 'start', 'title' => $title)) . str_replace("\n", " ", $inside) . $this->wiki->addToken($this->rule, array(
            'type' => 'end')) . "\n";
    }

    function insertBibitem($matches) {
        return $matches[2];
    }

}
