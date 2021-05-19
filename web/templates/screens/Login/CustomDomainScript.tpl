{$page->setLayout("Raw")}

{if isset($redir)}

	window.location = '{$redir}&url=' + encodeURIComponent(window.location);

{/if}

{if isset($redirIE)}

	{* only in IE try to redirect to set the short cookie *}

 	{literal}
	if (navigator.appName.indexOf('Internet Explorer') != -1) {
	{/literal}
		window.location = '{$redirIE}&url=' + encodeURIComponent(window.location);
	{literal}
	}
	{/literal}

{/if}
