<?php
class ASLanguageModule extends AccountBaseModule
{

    public function build($runData)
    {
        $user = $runData->getUser();
        $runData->contextAdd("lang", $user->getLanguage());
    }
}
