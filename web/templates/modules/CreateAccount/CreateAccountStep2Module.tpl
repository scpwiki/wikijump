{if isset($fromEmail)}
	<script type="text/javascript">
	var t2 = new OZONE.dialogs.SuccessBox(); t2.timeout=10000; t2.content="New account created!";t2.show();
	var originalUrl = '{$originalUrl}';
	var originalUrlForce = '{$originalUrlForce}';
	{literal}
	if(originalUrlForce){
		setTimeout(function(){
			//window.location.href = originalUrl;
		}, 2000);
	} else {
		setTimeout(function(){
			var url = '/auth:newaccount3';
			if(originalUrl){
				url = url + '?origUrl=' + encodeURIComponent(originalUrl);
			}
			window.location.href = url;
		}, 2000);
	}

	</script>
	{/literal}

{else}

	{loadmacro set="Forms"}
	<form id="reg2" action="dummy" method="post" onkeypress="return OZONE.utils.disableEnterKey(event)">
		<div class="error-block" id="ca-error-block" style="display: none"></div>
		<table class="form">
			<tr>
				<td>
					{t}Verification code{/t}:
				</td>
				<td>
					<input class="text" type="text" name="evercode" id="ca-evercode" size="20"/>
				</td>
			</tr>
		</table>

		{*<p>
			{$evcode}
		</p>*}

		<div class="buttons">
			<input type="button" class="button" onclick="Wikijump.modules.CreateAccountStep2Module.listeners.cancel(event)" value="{t}cancel{/t}"/>
			<input type="button" class="button" onclick="Wikijump.modules.CreateAccountStep2Module.listeners.nextClick(event)" value="{t}next{/t}"/>
		</div>
	</form>
{/if}
