<?php


 * @version $Id: lucene_search.php,v 1.1 2008/12/04 12:16:45 redbeard Exp $
 * @copyright Copyright (c) 2008-2020, Wikidot Inc., SCP Wiki Technical Team
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

require_once ("../php/setup.php");

$lucene = new Wikidot_Search_Lucene();
$lucene->createIndex();
$lucene->indexAllSitesVerbose();
