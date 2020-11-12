<?php
use DB\PagePeer;

class TemplateSourceModule extends SmartyModule
{

    public function build($runData)
    {
        $pageId = $runData->getParameterList()->getParameterValue("page_id");
        $page = PagePeer::instance()->selectByPrimaryKey($pageId);
        $source = $page->getSource();
//      /* Determine if it is a live template. */
//      if(preg_match(';%%content({[0-9]+})?%%;', $source)) {
//          $split = array();
//          $split = preg_split(';^=default={4,}$;sm', $source);
//          if(count($split) == 2){
//              /* Fine, there is some initial content. */
//              $source = trim($split[1]);
//          } else {
//              $source = null;
//          }
//      }
        $runData->contextAdd("source", $source);
    }
}
