<div id="membership-apply-box">
{if $reason == null}

<form id="membership-by-apply-form">
	<table class="form">
		<tr>
			<td>
				{t}Application text{/t}:
			</td>
			<td>
				<textarea name="comment" rows="5" cols="50" id="membership-by-apply-text"></textarea>
				<div class="sub" style="text-align: center;">
					(<span id="membership-by-apply-text-left"></span> {t}characters left{/t})
				</div>
			</td>
		</tr>
	</table>

	<div class="buttons">
		<input id="mba-apply" type="button" value="{t}apply!{/t}"/>
	</div>
</form>
{else}
	{if $reason=="not_logged"}
		<p>
			{t 1=$SERVICE_NAME}You should have a valid %1 account and be logged in order to apply for membership.{/t}
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
			{t}You can not apply.{/t}<br/>
			{if $reason=="not_enabled"}
				{t}Membership via application is not enabled for this site.{/t}
			{/if}
			{if $reason=="not_logged"}
				{t 1=$SERVICE_NAME}You should have a valid account at %1 and be logged in order to apply for membership.
				Please log in or create an account first.{/t}
			{/if}
			{if $reason=="already_applied"}
				{t}It seems you have already applied for membership.{/t}
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

