		<p>
			{t}To access account preferences and settings you should have a valid Wikidot account.{/t}
		</p>
		<table style="margin: 1em auto">
			<tr>
				<td style="text-align: center; padding: 1em">
					<div style="font-size: 180%; font-weight: bold;">
						<a href="javascript:;" onclick="WIKIDOT.page.listeners.loginClick(event)"
							>{t}Log in{/t}</a>
					</div>
					<p>	
						{t}if you already have a Wikidot account{/t}
					</p>
				</td>
				<td style="padding: 1em; font-size: 140%">
					{t}or{/t}
				</td>
				<td style="text-align: center; padding: 1em">
					<div style="font-size: 180%; font-weight: bold;">
						<a href="javascript:;"  onclick="WIKIREQUEST.createAccountSkipCongrats=true;WIKIDOT.page.listeners.createAccount(event)"
							>{t}Get a new Wikidot account{/t}</a>
					</div>
					<p>
						{t}It's free and safe, and only takes a second{/t}
					</p>
				</td>
			</tr>
		</table>
