<?php

namespace Wikidot\Modules\Search;

use Ozone\Framework\Database\Criteria;


use Ozone\Framework\SmartyModule;
use Wikijump\Models\User;

class UserSearchModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $query = trim($pl->getParameterValue("q"));

        $site = $runData->getTemp("site");

        if (!$query || $query == '') {
            return;
        }

        if (strlen($query)<3) {
            $runData->contextAdd("query", $query);
            $runData->contextAdd("errorMessage", _("Your query should be at least 3 characters long."));
            return;
        }

        // pagination

        $pageNumber = $pl->getParameterValue("p");
        if ($pageNumber == null || !is_numeric($pageNumber) || $pageNumber <1) {
            $pageNumber = 1;
        }
        $perPage = 10;

        // determine the mode: by email or by screenname/realname/slug

        if (strpos($query, '@')) {
            // email lookup mode
            $user = User::firstWhere('email', $query);
            $runData->contextAdd("user", $user);
            $runData->contextAdd("mode", "email");
            sleep(2);
        } else {
            // normal search. perform a regexp search at the moment.
            $qs = preg_split('/ +/', trim($query));

            $c = new Criteria();
            foreach ($qs as $q) {
                $csub = new Criteria();
                $csub->add("users.username", preg_quote($q), '~*');
                $csub->addOr("users.slug", preg_quote($q), '~*');
                $csub->addOr("users.real_name", preg_quote($q), '~*');

                $c->addCriteriaAnd($csub);
            }

            $c->addJoin("user_id", "users.id");

            $limit = $perPage*2+1;
            $offset = ($pageNumber - 1)*$perPage;

            $c->setLimit($limit, $offset);

            $res = null;
            $counted = count($res);

            $pagerData = array();
            $pagerData['current_page'] = $pageNumber;
            if ($counted >$perPage*2) {
                $knownPages=$pageNumber + 2;
                $pagerData['known_pages'] = $knownPages;
            } elseif ($counted>$perPage) {
                $knownPages=$pageNumber + 1;
                $pagerData['total_pages'] = $knownPages;
            } else {
                $totalPages = $pageNumber;
                $pagerData['total_pages'] = $totalPages;
            }

            $res = array_slice($res, 0, $perPage);
        }

        $runData->contextAdd("pagerData", $pagerData);

        $runData->contextAdd("users", $res);
        $runData->contextAdd("countResults", count($res));
        $runData->contextAdd("query", $query);
        $runData->contextAdd("encodedQuery", urldecode($query));
        $runData->contextAdd("queryEncoded", urlencode($query));
    }
}
