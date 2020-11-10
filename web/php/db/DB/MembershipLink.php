<?php


namespace DB;





/**
 * Object Model class.
 *
 */
class MembershipLink extends MembershipLinkBase
{

    public function getTypeDescription()
    {

        switch ($this->getType()) {
            case 'INTERNAL_INVITATION':
                return _("invitation");
            case 'EMAIL_INVITATION':
                return _("email invitation");
            case 'APPLICATION_ACCEPTED':
                return _("accepted membership application");
            case 'BY_PASSWORD':
                return _("by password");
        }
    }
}
