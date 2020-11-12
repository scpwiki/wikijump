<?php
// vim: set expandtab tabstop=4 shiftwidth=4 softtabstop=4:
/**
 * Image rule end renderer for Xhtml
 *
 * PHP versions 4 and 5
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Paul M. Jones <pmjones@php.net>
 * @author Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    $Id$
 * @link       http://pear.php.net/package/Text_Wiki
 */

/**
 * This class inserts an image in XHTML.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Paul M. Jones <pmjones@php.net>
 * @author Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_Form extends Text_Wiki_Render {

    /**
    *
    * Renders a token into text matching the requested format.
    *
    * @access public
    *
    * @param array $options The "options" portion of the token (second
    * element).
    *
    * @return string The text rendered from the token options.
    *
    */

    function token($options)
    {
        if (isset($options['begin'])) {
            return '<table class="form-view">';
        }
        if (isset($options['end'])) {
            return '</table>';
        }

        $field = Wikidot_Form_Field::field($options['field']);
        $h_label = htmlspecialchars($options['field']['label']);
        $h_value = $field->renderView();

        $output = '<tr class="form-view-field">';
        $output .= "<td class=\"form-view-field-label\">$h_label:</td>";
        $output .= "<td class=\"form-view-field-value\">$h_value</td>";
        $output .= '</tr>';

        return $output;
    }
}
