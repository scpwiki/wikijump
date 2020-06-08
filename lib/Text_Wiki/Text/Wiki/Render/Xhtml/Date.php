<?php
/**
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    $Id$
 * @link       http://pear.php.net/package/Text_Wiki
 */

/**
 * Iframe element.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_Date extends Text_Wiki_Render {

    function token($options){
       $timestamp = $options['timestamp'];
       $format = $options['format'];
       if($format){
       	$format = '|'.$format;
       }
       	$output = '<span class="odate">' . $timestamp . $format . '</span>';

        return $output;
    }
}
