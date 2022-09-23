<?php
declare(strict_types=1);

namespace Wikijump\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\Http\Response;
use Wikijump\Models\Settings;

class SettingsController extends Controller
{
    /**
     * Display a listing of the resource.
     *
     * @return Response
     */
    public function index(): Response
    {
        //
    }

    /**
     * Show the form for creating a new resource.
     *
     * @return Response
     */
    public function create(): Response
    {
        //
    }

    /**
     * Store a newly created resource in storage.
     *
     * @param Request $request
     * @return Response
     */
    public function store(Request $request): Response
    {
        //
    }

    /**
     * Display the specified resource.
     *
     * @param Settings $settings
     * @return Response
     */
    public function show(Settings $settings): Response
    {
        //
    }

    /**
     * Show the form for editing the specified resource.
     *
     * @param Settings $settings
     * @return Response
     */
    public function edit(Settings $settings): Response
    {
        //
    }

    /**
     * Update the specified resource in storage.
     *
     * @param Request $request
     * @param Settings $settings
     * @return Response
     */
    public function update(Request $request, Settings $settings): Response
    {
        //
    }

    /**
     * Remove the specified resource from storage.
     *
     * @param Settings $settings
     * @return Response
     */
    public function destroy(Settings $settings): Response
    {
        //
    }
}
