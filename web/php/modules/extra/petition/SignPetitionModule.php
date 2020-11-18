<?php
use DB\PetitionCampaignPeer;
use DB\PetitionSignaturePeer;

class SignPetitionModule extends SmartyModule
{

    public function build($runData)
    {

        require(WIKIJUMP_ROOT.'/php/unclassified/country_codes.php');
        $runData->contextAdd("coutryCodes", $iso3166_country_codes);

        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $id = $pl->getParameterValue("id");

        if (!$id) {
            throw new ProcessException(_("The campaign identifier is not valid."));
        }

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("deleted", false);
        $c->add("identifier", $id);

        $camp = PetitionCampaignPeer::instance()->selectOne($c);

        if (!$camp) {
            throw new ProcessException(_("The campaign cannot be found."));
        }

        if (!$camp->getActive()) {
            throw new ProcessException(_("This petition campaign is paused."));
        }

        $runData->contextAdd("campaign", $camp);

        $confirm = $pl->getParameterValue("confirm");

        if ($confirm) {
//          // working in the CONFIRMATION mode!
//
//          // get the petition
//
//

            $db = Database::connection();
            $db->begin();

            // get the petition
            $hash = $confirm;
            $c = new Criteria();
            $c->add("campaign_id", $camp->getCampaignId());
            $c->add("confirmation_hash", $hash);
            $pet = PetitionSignaturePeer::instance()->selectOne($c);

            if (!$pet) {
                throw new ProcessException(_("The petition signature cannot be found."));
            }
            if ($pet->getConfirmed()) {
                throw new ProcessException(_("This signature has been already confirmed."));
            }

            // confirm it and redirect to a "thank you" page or display "thank you".

            $pet->setConfirmed(true);
            $pet->setConfirmationUrl(null);

            $pet->save();

            $camp->updateNumberSignatures();
            $camp->save();

            $db->commit();

            $thankYouPage = $camp->getThankYouPage();
            if ($thankYouPage) {
                // simply REDIRECT!
                header("HTTP/1.1 301 Moved Permanently");
                header("Location: /".$thankYouPage);
                exit();
            } else {
                $runData->setModuleTemplate("extra/petition/SignatureConfirmedModule");
            }
        }
    }
}
