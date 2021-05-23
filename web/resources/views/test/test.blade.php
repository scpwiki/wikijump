@extends('layouts.legacy')

@section('content')
    @if($pageNotExists == true)
        <p>
            The page {{$wikiPageName}} you want to access does not exist.
        </p>
        <ul>
            <li><a href="javascript:;" onclick="Wikijump.page.listeners.editClick(event)">create page</a></li>
        </ul>
    @else
        {!! $pageContent !!}
    @endif
@endsection
