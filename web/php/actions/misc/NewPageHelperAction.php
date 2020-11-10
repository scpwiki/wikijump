<?php
use DB\PagePeer;

class NewPageHelperAction extends SmartyAction
{

    public function perform($r)
    {
    }

    public function createNewPageEvent($runData)
    {
        // this just checks if page exists and if the user has permissions to create.
        // returns cleaned name.

        $pl = $runData->getParameterList();
        $pageName = trim($pl->getParameterValue("pageName"));
        $categoryName = trim($pl->getParameterValue("categoryName"));
        $format =  trim($pl->getParameterValue("format"));
        $autoincrement = $pl->getParameterValue('autoincrement');

        $templateId = $pl->getParameterValue("template");

        $site = $runData->getTemp("site");

        if (strlen($pageName) === 0) {
            $runData->ajaxResponseAdd("status", "no_name");
            $runData->ajaxResponseAdd("message", "You should provide a page name.");
            return;
        }

        // check if use a title too
        //if(WDStringUtils::toUnixName($pageName) != $pageName){
            $pageTitle = $pageName;
        //}

        if ($format) {
            $m = false;
            $m = @preg_match($format, $pageName);

            if ($m !== false && $m === 0) {
                throw new ProcessException(_("The page name is not in the required format."));
            }
        }

        if ($autoincrement) {
            $unixName = $categoryName . ':autoincrementpage';
        } else {
            $unixName = WDStringUtils::toUnixName($categoryName.':'.$pageName);
        }

        $page = PagePeer::instance()->selectByName($site->getSiteId(), $unixName);
        if ($page != null) {
            $runData->ajaxResponseAdd("status", "page_exists");
            $runData->ajaxResponseAdd("message", "The page <em>".$unixName."</em> already exists." .
                    ' <a href="/'.$unixName.'">Jump to it</a> if you wish.');
            return;
        }

        if ($templateId) {
            $templatePage = PagePeer::instance()->selectByPrimaryKey($templateId);
            if (!$templatePage || !preg_match("/^template:/", $templatePage->getUnixName())) {
                throw new ProcessException("Error selecting the template");
            }

            $runData->ajaxResponseAdd("templateId", $templateId);
        }

        $runData->ajaxResponseAdd("unixName", $unixName);
        if ($pageTitle) {
            $runData->ajaxResponseAdd("pageTitle", $pageTitle);
        }
    }
}
