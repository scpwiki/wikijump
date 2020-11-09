<?php
/**
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
 * Parses for advanced table.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Paul M. Jones <pmjones@php.net>
 * @author Michal Frackowiak
 *
 */

class Text_Wiki_Parse_Tableadv extends Text_Wiki_Parse {

    public $regex =     ';' . 
                        '\n\[\[table' .                        # Start a table after a new line
                        '(\s.*?)?\]\]' .                       # Allow parameters on the table
                        '(\s*' . 
                            '(?:\[\[row' .                     # Start a row
                                '(?:\s[^\]]*)?' .              # Allow parameters on the row
                                '\]\]\s*' . 
                                '(?:\[\[(column|col|cell)' .   # Start a column or cell
                                    '(?:\s[^\]]*)?\]\]' .      # Allow parameters on the column or cell
                                    '(?:(?R)|.)*?' .           # Cell contents: another table, or anything else
                                '\[\[/(column|col|cell)' .     # End the column or cell
                                '\]\]\s*)+' .                  # Allow at least one column or cell
                            '\[\[/row\]\]\s*)+' .              # Allow at least one row
                        ')' . 
                        '\[\[/table\]\]\n' .                   # Force a new line after the table
                        ';sxi';

    private $_tmpSource = null;

    function process(&$matches) {
        $content = $matches[2];
        $attr = $this->getAttrs(trim($matches[1]));
        $options = array();
        if ($attr['class']) {
            $options['class'] = $attr['class'];
        }
        if ($attr['style']) {
            $options['style'] = $attr['style'];
        }

        // look for nested tables - TODO


        $content = preg_replace_callback(
            $this->regex, array(&$this, 'process'), $content);

        // look for rows
        $content = preg_replace_callback(   ';' . 
                                            '(?:\n)?' .                      # Allow an optional new line
                                            '\[\[row' .                      # Start a new row
                                            '(\s[^\]]*)?' .                  # Allow parameters on row
                                            '\]\]\s*' . 
                                            '(' . 
                                                '(' . 
                                                    '\[\[' . 
                                                    '(column|col|cell)' .    # Start a new column or cell
                                                    '(\s[^\]]*)?' .          # Allow parameters on column or cell
                                                    '\]\].*?' . 
                                                    '' .                  # No content is permitted
                                                    '\[\[/' . 
                                                    '(column|col|cell)' .    # Close column or cell
                                                    '\]\]\s*' . 
                                                ')+' . 
                                            ')\[\[/row\]\]' .                # Close row
                                            '(?:\n)?' .                      # Allow an optional new line
                                            ';msix',
            array($this, '_handleRow'), $content);

        $start = $this->wiki->addToken($this->rule,
            array_merge($options, array('type' => 'start')));
        $end = $this->wiki->addToken($this->rule, array('type' => 'end'));
        return "\n\n" . $start . $content . $end . "\n\n";
    }

    function parse() {

        $this->wiki->source = preg_replace_callback(
            $this->regex, array(&$this, 'process'), $this->wiki->source);
    }

    private function _handleRow($matches) {
        $content = $matches[2];
        $attr = $this->getAttrs(trim($matches[1]));

        $content = preg_replace_callback(   ';' . 
                                            '(?:\n)?' .                                  # Optional new line
                                            '\[\[(?:column|col|cell)(\s[^\]]*)?\]\]' .   # Opening column or cell
                                                '(.*?)' .                                # Content (anything)
                                            '\[\[/(column|col|cell)\]\]' .               # Closing column or cell
                                            '(?:\n)?' .                                  # Optional new line
                                            ';msix',
            array($this, '_handleCell'), $content);
        $options = array();
        if ($attr['class']) {
            $options['class'] = $attr['class'];
        }
        if ($attr['style']) {
            $options['style'] = $attr['style'];
        }
        $start = $this->wiki->addToken($this->rule, array_merge($options, array(
            'type' => 'rowStart')));
        $end = $this->wiki->addToken($this->rule, array(
            'type' => 'rowEnd'));
        return $start . $content . $end;

    }

    private function _handleCell($matches) {
        $content = $matches[2];
        $attr = $this->getAttrs(trim($matches[1]));
        $options = array();
        if ($attr['class']) {
            $options['class'] = $attr['class'];
        }
        if ($attr['style']) {
            $options['style'] = $attr['style'];
        }

        // newlines if no paragraps inside
        if (strpos($content, "\n\n") === false) {
            $content = trim($content);
        } else {
            $content = "\n\n" . $content . "\n\n";
        }

        $start = $this->wiki->addToken($this->rule, array_merge($options, array(
            'type' => 'cellStart')));
        $end = $this->wiki->addToken($this->rule, array(
            'type' => 'cellEnd'));
        return $start . $content . $end;
    }
}
