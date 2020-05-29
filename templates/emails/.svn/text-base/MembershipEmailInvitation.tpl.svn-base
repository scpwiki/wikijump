Hello {$name},

{if $message2}
{$message2|wordwrap}

The original message is included below:

-----------------------------------------

{/if}
{$user->getNickName()} {if $profile->getRealName()}({$profile->getRealName()}){/if} would like to invite you
to join members of the wiki website "{$site->getName()}"
created at {$SERVICE_NAME} and located at the address 
http://{$site->getDomain()|escape}.
{if $message!=""}

{$message|wordwrap}{/if}	

Signing up is easy and takes less than a minute. If you already have
an account at {$SERVICE_NAME},
you will be able to join the mentioned
Site. 

To proceed or learn more click the follow link:
http://{$URL_HOST}/invitation/hash/{$hash}

See you

{$user->getNickName()|escape} {if $profile->getRealName()}({$profile->getRealName()}){/if}


P.S. If you do not want to accept this invitation - just ignore it.
If you believe this invitation is an abuse - please report it to:

{$SUPPORT_EMAIL}