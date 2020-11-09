<?php

use DB\PagePeer;
use Wikijump\Form;
use Wikijump\Template;

class CodeblockExtractor
{

    protected $mimeType = null;
    protected $contents = "";
    protected $treatAsTemplate = false;
    protected $templateVariables = array();

    protected $mimeMap = array(
        "css"   => "text/css",
        "html"  => "text/html",
    );

    public function __construct($site, $pageName, $codeblockNo = 1, $templateVars = null)
    {
        try {
            $codeblockNo = (int) $codeblockNo;
            if ($codeblockNo < 1) {
                $codeblockNo = 1;
            }

            $page = PagePeer::instance()->selectByName($site->getSiteId(), $pageName);

            if ($page == null) {
                throw new ProcessException("No such page");
            }
            // page exists!!! wooo!!!

            $source = $page->getSource();
            /* Get code block. */

            $regex = ';^\[\[code(\s\V+)?\]\]((?>[^[]+|(?R)|.)+?)\[\[\/code\]\];ms';

            $allMatches = array();
            preg_match_all($regex, $source, $allMatches);

            if (count($allMatches[2]) < $codeblockNo) {
                throw new ProcessException('No valid codeblock found.');
            }

            $code = $allMatches[2][$codeblockNo - 1];
            if ($allMatches[1][$codeblockNo - 1]) {
                $params = $allMatches[1][$codeblockNo - 1];
                $m = array();
                $type = null;
                if (preg_match('/type="([^"]+)"/', $params, $m)) {
                    $type = strtolower($m[1]);
                }
                if (array_key_exists($type, $this->mimeMap)) {
                    $this->mimeType = $this->mimeMap[$type];
                }
            }

            $code = trim($code) . "\n";

            if (is_array($templateVars)) {
                $this->contents = $this->renderFromTemplate($code, $templateVars);
            } else {
                $this->contents = $code;
            }
        } catch (Exception $e) {
            $this->contents = $e->getMessage();
        }
    }

    public function getContents()
    {
        return $this->contents;
    }

    public function getMimeType()
    {
        if ($this->mimeType) {
            return $this->mimeType;
        }
        return "text/plain";
    }

    public function renderFromTemplate($template, $extValues)
    {
        $template = "\n$template\n";
        $template_parts = explode("\n---\n", $template);

        // form definition is the YAML document before the first "---"
        $form_def = array_shift($template_parts);

        // Wikijump (DTL) template is the rest
        $template = trim(implode("\n---\n", $template_parts));

        $form = Form::fromYaml($form_def);
        $context = $form->computeValues($extValues);

        // render the template
        $w_template = new Template($template);
        return $w_template->render($context);
    }
}
