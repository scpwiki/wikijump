<?php

namespace Wikidot\Modules\XList;


require_once(WIKIJUMP_ROOT . '/php/Modules/List/NextPageModule.php');

class PreviousPageModule extends NextPageModule
{

    protected $orderType = 'Desc';
    protected $listPagesParam = 'previousBy';
}
