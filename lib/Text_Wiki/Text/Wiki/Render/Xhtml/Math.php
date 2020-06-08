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
 * Math equation.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_Math extends Text_Wiki_Render {

    public $conf = array();

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

    function token($options) {
        $content = $options['content'];
        $type = $options['type'];
        $hashcode = md5($content . '..' . $type);
        $runData = Ozone::getRunData();
        $site = $runData->getTemp('site');
        $dir = $site->getLocalFilesPath() . '/math/eqs';
        if (!file_exists($dir)) {
            mkdirfull($dir);
        }

        $tmpDir = WIKIDOT_ROOT . '/tmp/math';
        if (!file_exists($tmpDir)) {
            mkdirfull($tmpDir);
        }

        $imgFile = $hashcode . '.png';
        if (!file_exists($dir . '/' . $imgFile)) {
            $renderer = new LatexRenderer();
            $renderer->setTmpDir($tmpDir);
            $renderer->setOutputDir($dir);

            if ($type == 'eqnarray') {
                $content2 = "\\begin{eqnarray*}\n" . $content . "\n\\end{eqnarray*}";
            } else {
                $content2 = "\\begin{equation*}\n" . $content . "\n\\end{equation}";
            }
            $renderer->render($content2, $hashcode);
        }

        if (!file_exists($dir . '/' . $imgFile)) {
            return '<div class="error-block">' . _('The equation has not been processed correctly. Most prabably it has syntax error(s).') . '</div>';
        }

        $label = $options['label'];
        $idPrefix = $this->getConf("id_prefix");
        $idString = ' id="equation-' . $idPrefix . $options['id'] . '" ';
        $equationNumberLabel = '<span class="equation-number">(' . $options['id'] . ')</span>';
        $out = '<div class="math-equation"' . $idString . '><img src="/local--math/eqs/' . $imgFile . '" alt="' . htmlentities($content) . '" /></div>';

        return $equationNumberLabel . $out;
    }
}
