<?php


 * @version $Id: ListPagesModule.php,v 1.10 2008/05/27 13:27:06 redbeard Exp $
 * @copyright Copyright (c) 2008-2020, Wikidot Inc., SCP Wiki Technical Team
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */


require_once(WIKIDOT_ROOT . '/php/modules/list/NextPageModule.php');

class PreviousPageModule extends NextPageModule
{

    protected $orderType = 'Desc';
    protected $listPagesParam = 'previousBy';
}
