<?php
use DB\PetitionCampaignPeer;
use DB\PetitionSignaturePeer;

class PetitionDataDownload extends SmartyScreen
{

    public function isAllowed($runData)
    {
        WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));
        return true;
    }

    public function build($runData)
    {
    }

    public function render($runData)
    {

        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $campaignId = $pl->getParameterValue("campaignId");

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("deleted", false);
        $c->add("campaign_id", $campaignId);

        $camp = PetitionCampaignPeer::instance()->selectOne($c);

        if (!$camp) {
            throw new ProcessException(_("The campaign can not be found."));
        }

        $out = '';

        $header = array();

        $header[] = "First name";
        $header[] = "Last name";
        $header[] = "Email";

        if ($camp->getCollectAddress()) {
            $header[] = "Address1";
            $header[] = "Address2";
        }
        if ($camp->getCollectCity()) {
            $header[] = "City";
        }
        if ($camp->getCollectState()) {
            $header[] = "State";
        }
        if ($camp->getCollectZip()) {
            $header[] = "Zip";
        }
        if ($camp->getCollectCountry()) {
            $header[] = "Country";
            $header[] = "CountryCode";
        }
        $header[] = "Date";
        if ($camp->getCollectComments()) {
            $header[] = "Comments";
        }

        $out .= $this->formatCsvRow($header);

        // get the signatures now

        $c = new Criteria();
        $c->add("campaign_id", $camp->getCampaignId());
        $c->add("confirmed", true);
        $c->addOrderAscending("signature_id");
        $signatures = PetitionSignaturePeer::instance()->select($c);

        $q = "SELECT * FROM petition_signature WHERE campaign_id={$camp->getCampaignId()} AND confirmed=TRUE ORDER BY signature_id";
        $db = Database::connection();
        $rr = $db->query($q);

        while ($r = $rr->nextRow()) {
            $row = array();
            $row[] = $r['first_name'];
            $row[] = $r['last_name'];
            $row[] = $r['email'];

            if ($camp->getCollectAddress()) {
                $row[] = $r['address1'];
                $row[] = $r['address2'];
            }
            if ($camp->getCollectCity()) {
                $row[] = $r['city'];
            }
            if ($camp->getCollectState()) {
                $row[] = $r['state'];
            }
            if ($camp->getCollectZip()) {
                $row[] = $r['zip'];
            }
            if ($camp->getCollectCountry()) {
                $row[] = $r['country'];
                $row[] = $r['country_code'];
            }
            $row[] = date($r['date']);
            if ($camp->getCollectComments()) {
                $row[] = $r['comments'];
            }

            $out .= $this->formatCsvRow($row);
        }

//
//

        header('Content-type: text/plain;');
        return $out;
    }

    private function formatCsvRow($row)
    {
        // $row is an array.
        foreach ($row as &$value) {
            if (preg_match("/[,\"\n]/", $value)) {
                $value = str_replace('"', '""', $value);
                $value = '"'.$value.'"';
            }
        }
        return implode(',', $row)."\n";
    }
}
