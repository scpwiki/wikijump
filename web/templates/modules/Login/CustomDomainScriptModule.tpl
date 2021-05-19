{if isset($useCustomDomainScript)}
	<script type="text/javascript" src="http{if isset($useCustomDomainScriptSecure)}s{/if}://{$URL_HOST}/default__flow/Login__CustomDomainScript?site_id={$site->getSiteId()}"></script>
{/if}
