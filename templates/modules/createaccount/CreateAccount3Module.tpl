{loadmacro set="Forms"}
<div class="owindow" style="width: 50em">
	<div class="title">
		{t}Create account: finished!{/t}
	</div>
	<div class="content">
		<h1>{t}Congratulations{/t}, {$user->getNickName()|escape}!</h1>
		
		<p>
			{t 1=$SERVICE_NAME}And welcome to %1! Now, as a registered member,
			you can explore dozens of great Wikidot features, unleash your
			creativity or efficiently use Wikidot.{/t}
		</p>
			
		<h2>{t}Where to go now?{/t}</h2>
		
		<ul>
			<li>
				{t escape=no}You could go to <a href="http://{$URL_HOST}/account:you">your account panel</a>
				to{/t}: 
				<ul>
					<li>{t}configure your profile - load buddy icon (avatar), provide more
						personal information about yourself{/t},</li>
					<li>{t}configure extra settings for your account{/t},</li>
					<li>{t}learn how to watch sites, pages, use private messaging etc.{/t}</li>
				</ul>
			</li>
			<li>
				{t 1=$URL_HOST escape="no"}<a href="http://%1/new-site">Create a new wiki</a>, invite people to participate, create discussion fora etc.{/t}
			</li>
			
		</ul>
		
	</div>
	
	<div class="button-bar">
		<a href="javascript:;" onclick="window.location.reload()"/>close message</a>
	</div>
</div>
