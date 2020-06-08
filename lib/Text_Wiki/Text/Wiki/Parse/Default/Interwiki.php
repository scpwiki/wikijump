<?php

/**
 *
 * Parses for interwiki links.
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
 * Parses for interwiki links.
 *
 * This class implements a Text_Wiki_Parse to find source text marked as
 * an Interwiki link.  See the regex for a detailed explanation of the
 * text matching procedure; e.g., "InterWikiName:PageName".
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Paul M. Jones <pmjones@php.net>
 * @author Michal Frackowiak
 *
 */

class Text_Wiki_Parse_Interwiki extends Text_Wiki_Parse {

    // double-colons wont trip up now
    public $regex = '([A-Za-z0-9_\.]+):((?!:)[A-Za-z0-9_\/=&~#.:;\-\+]+)';

    /**
     *
     * Parser.  We override the standard parser so we can
     * find both described interwiki links and standalone links.
     *
     * @access public
     *
     * @return void
     *
     */

    function parse() {
        // standalone interwiki links
        $tmp_regex = '/\[(' . $this->regex . ')\]/';
        $this->wiki->source = preg_replace_callback($tmp_regex, array(
            &$this, 'process'), $this->wiki->source);

        // described interwiki links
        $tmp_regex = '/\[' . $this->regex . ' (.+?)\]/';
        $this->wiki->source = preg_replace_callback($tmp_regex, array(
            &$this, 'processDescr'), $this->wiki->source);

    }

    /**
     *
     * Generates a replacement for the matched standalone interwiki text.
     * Token options are:
     *
     * 'site' => The key name for the Text_Wiki interwiki array map,
     * usually the name of the interwiki site.
     *
     * 'page' => The page on the target interwiki to link to.
     *
     * 'text' => The text to display as the link.
     *
     * @access public
     *
     * @param array &$matches The array of matches from parse().
     *
     * @return A delimited token to be used as a placeholder in
     * the source text, plus any text priot to the match.
     *
     */

    function process(&$matches) {
        $t = substr($matches[1], strpos($matches[1], ':') + 1);
        $options = array('site' => $matches[2],
            'page' => $matches[3], 'text' => $t);

        // check if site exists NOW.


        $sites = $this->getConf('sites');

        if ($sites[$matches[2]] == null) {
            return $matches[0];
        } else {
            return $this->wiki->addToken($this->rule, $options);
        }
    }

    /**
     *
     * Generates a replacement for described interwiki links. Token
     * options are:
     *
     * 'site' => The key name for the Text_Wiki interwiki array map,
     * usually the name of the interwiki site.
     *
     * 'page' => The page on the target interwiki to link to.
     *
     * 'text' => The text to display as the link.
     *
     * @access public
     *
     * @param array &$matches The array of matches from parse().
     *
     * @return A delimited token to be used as a placeholder in
     * the source text, plus any text priot to the match.
     *
     */

    function processDescr(&$matches) {
        $options = array('site' => strtolower($matches[1]),
            'page' => $matches[2],
            'text' => $matches[3]);

        return $this->wiki->addToken($this->rule, $options);
    }
}
