<?php

namespace Wikidot\Utils;

use Ozone\Framework\Ozone;
use Ozone\Framework\Screen;

class FeedScreen extends Screen
{

    protected $requiresAuthentication = false;

    public function getRequiresAuthentication()
    {
        return $this->requiresAuthentication;
    }

    public function render($runData)
    {

        $this->build($runData);

        $smarty = Ozone::getSmarty();

        // put context into context

        $context = $runData->getContext();
        if ($context !== null) {
            foreach ($context as $key => $value) {
                $smarty->assign($key, $value);
            }
        }

        $templateFile = WIKIJUMP_ROOT.'/templates/screens/Feed/FeedTemplate.tpl';
        $out = $smarty->fetch($templateFile);

        return $out;
    }
}
