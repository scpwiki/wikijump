{$firstName} {$lastName}!

{t 1=$campaignName 2=$url 3=$hash 4=$siteName}Thanks a lot for signing the "%1" petition.  To ensure
that it's really you, and not someone else using your email address,
please click on the following link to confirm your signature:

http://%2/confirm/%3

Many thanks,

The "%4" team.{/t}

----------------------------------------------------------------------------

{t}The data you have entered is:{/t}

{t}First name{/t}: {$sig->getFirstName()}
{t}Last name{/t}: {$sig->getLastName()}
{if $campaign->getCollectAddress()}
{t}Address{/t}: {$sig->getAddress1()}{if $sig->getAddress2() != ""}, {$sig->getAddress2()}{/if}
{/if}
{if $campaign->getCollectCity()}
{t}City{/t}: {$sig->getCity()}
{/if}
{if $campaign->getCollectState()}
{t}State/Region{/t}: {$sig->getState()}
{/if}
{if $campaign->getCollectZip()}
{t}Zip/postal code{/t}: {$sig->getZip()}
{/if}
{if $campaign->getCollectCountry()}
{t}Country{/t}: {$sig->getCountry()}
{/if}
{if $campaign->getCollectComments() && $sig->getComments()}
{t}Your comments{/t}:
{textformat}{$sig->getComments()}{/textformat}
{/if}


{t}If you have no idea what this email is about this means someone
has entered your email address, most probably by mistake.{/t}