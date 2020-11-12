<?php

require_once(WIKIJUMP_ROOT . '/php/modules/list/NextPageModule.php');

class PreviousPageModule extends NextPageModule
{

    protected $orderType = 'Desc';
    protected $listPagesParam = 'previousBy';
}
