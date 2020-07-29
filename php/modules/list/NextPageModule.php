<?php

require_once(WIKIJUMP_ROOT . '/php/modules/list/ListPagesModule.php');

class NextPageModule extends ListPagesModule
{

    protected $orderType = 'Asc';
    protected $listPagesParam = 'nextBy';

    /**
     *
     * @param $runData RunData
     */
    public function render($runData)
    {
        $runData->setModuleTemplate("list/ListPagesModule");

        $pl = $runData->getParameterList();
        $by = $pl->getParameterValue('by');
        $pl->delParameter('by');

        if ($by == 'title') {
            $by = 'title';
            $order = "title" . $this->orderType;
        } else {
            $by = 'page_id';
            $order = "dateCreated" . $this->orderType;
        }

        $pl->addParameter($this->listPagesParam, $by, 'MODULE');
        $pl->addParameter('order', $order, 'MODULE');
        $pl->addParameter('limit', 1, 'MODULE');

        return parent::render($runData);
    }
}
