<div id="membership-by-password-box">
{if $reason == null}
<div id="mbp-error" class="error-block" style="display: none;">
	<div>
		{t}The password is not valid.{/t}
	</div>
</div>

<form id="membership-by-password-form" onsubmit="return false;" action="dummy.html" method="get">
	<table class="form">
		<tr>
			<td>
				{t}Password{/t}:
			</td>
			<td>
				<input class="text" type="password" name="password" size="40" maxlength="50"/><br/>
			</td>
		</tr>
	</table>
	<div class="buttons">
		<input id="mbp-apply" type="button" value="{t}apply!{/t}" onclick="WIKIDOT.modules.MembershipByPasswordModule.listeners.apply(event)"/>
	</div>
</form>
{else}
	{if $reason=="not_logged"}
		<p>
			{t 1=$SERVICE_NAME}You should have a valid account at %1 and be logged in order to enter the membership password.{/t}
		</p>
		<table style="margin: 1em auto">
			<tr>
				<td style="text-align: center; padding: 1em">
					<div style="font-size: 180%; font-weight: bold;">
						<a href="javascript:;" onclick="WIKIDOT.page.listeners.loginClick(event)"
							>{t}log in{/t}</a>
					</div>
					<p>	
						{t 1=$SERVICE_NAME}if you already have an account at %1{/t}
					</p>
				</td>
				<td style="padding: 1em; font-size: 140%">
					{t}or{/t}
				</td>
				<td style="text-align: center; padding: 1em">
					<div style="font-size: 180%; font-weight: bold;">
						<a href="javascript:;"  onclick="WIKIDOT.page.listeners.createAccount(event)"
							>{t}create a new account{/t}</a>
					</div>
				</td>
			</tr>
		</table>
	{else}
		<div class="error-block">
			You can not apply.<br/>
			{if $reason=="not_enabled"}
				{t}Membership via password is not enabled for this site.{/t}
			{/if}
			{if $reason=="already_member"}
				{t}It seems you already are a member of this site.{/t}
			{/if}
			{if $reason=="already_admin"}
				{t}It seems you already are an admin of this site.{/t}
			{/if}
		</div>
	{/if}
{/if}

</div>