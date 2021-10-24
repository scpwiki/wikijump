@guest
<a href="{{ route('register') }}">{{ _('Register') }}</a> {{ _('or') }}
<a href="{{ route('login') }}">{{ _('Log In') }}</a>
@endguest
@auth
<span class="printuser">
<a href="/">
    <img class="small" src="{{ Auth::user()->avatar() }}" alt="avatar" style="background-image:url('/user--karma/{{ Auth::id() }}')" />
    {{ Auth::user()->username }}
</a>
</span>
<a href="/account:you">{{ _('my account') }}</a>
<a id="account-topbutton" href="javascript:;">&nabla;</a>
<div id="account-options">
    <ul>
        <li><a href="/account:you">{{ _('account summary') }}</a></li>
        <li><a href="/account:you/start/messages">{{ _('private messages') }}</a></li>
        <li><a href="/account:you/start/contacts">{{ _('my contacts') }}</a></li>
        <li><a href="/account:you/start/notifications">{{ _('notifications') }}</a></li>
        <li><a href="/account:you/start/watched-changes">{{ _('watched pages') }}</a></li>
        <li><a href="/account:you/start/watched-forum">{{ _('watched discussions') }}</a></li>
        <li><a href="{{ route('logout') }}">{{ _('Log Out') }}</a></li>
    </ul>
</div>
@endauth

