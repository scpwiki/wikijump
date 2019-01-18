<div class="owindow" style="width: 60%">
	<div class="title">
		{t}Add a contact{/t}
	</div>
	<div class="content">
		{ltext lang="en"}
		<h1>Adding <em>{$user->getNickName()|escape}</em> as a contact</h1>
		<p>
			Are you sure you want to add {printuser user=$user image=true} 
			as your new contact?
		</p>
		<p>
			Note: if you add <em>{$user->getNickName()|escape}</em> as a contact you
			will reveal your email address to this user.
		</p>
		{/ltext}
		{ltext lang="pl"}
		<h1>Dodać użytkownika<em>{$user->getNickName()|escape}</em> jako kontakt?</h1>
		<p>
			Jesteś pewien, że chcesz dodać użytkownika {printuser user=$user image=true} 
			do swojej listy kontaktów?
		</p>
		<p>
			Uwaga: w ten sposób umożliwisz mu/jej zobaczenie Twojego adresu email.
		</p>
		{/ltext}
	</div>
	<div class="button-bar">
		<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}cancel{/t}</a>
		<a href="javascript:;" onclick="WIKIDOT.modules.UserAddToContacts.listeners.addContact(event, {$user->getUserId()})">{t}yes, add please!{/t}</a>
	</div>
</div>