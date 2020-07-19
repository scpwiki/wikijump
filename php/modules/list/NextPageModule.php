<?php


 * @version $Id: ListPagesModule.php,v 1.10 2008/05/27 13:27:06 redbeard Exp $
 * @copyright Copyright (c) 2008-2020, Wikidot Inc., SCP Wiki Technical Team
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */


require_once(WIKIDOT_ROOT . '/php/modules/list/ListPagesModule.php');

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
