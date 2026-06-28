use std::borrow::Cow;

use serde_json::Value;

use crate::client::{OandaClient, OandaResult};

pub struct AccountsApi<'a> {
    client: &'a OandaClient,
}

pub struct AccountApi<'a> {
    client: &'a OandaClient,
    account_id: Option<Cow<'a, str>>,
}

pub struct InstrumentApi<'a> {
    client: &'a OandaClient,
    instrument: Cow<'a, str>,
}

pub struct OrdersApi<'a> {
    client: &'a OandaClient,
    account_id: Option<Cow<'a, str>>,
}

pub struct TradesApi<'a> {
    client: &'a OandaClient,
    account_id: Option<Cow<'a, str>>,
}

pub struct PositionsApi<'a> {
    client: &'a OandaClient,
    account_id: Option<Cow<'a, str>>,
}

pub struct PricingApi<'a> {
    client: &'a OandaClient,
    account_id: Option<Cow<'a, str>>,
}

pub struct TransactionsApi<'a> {
    client: &'a OandaClient,
    account_id: Option<Cow<'a, str>>,
}

impl OandaClient {
    pub fn accounts(&self) -> AccountsApi<'_> {
        AccountsApi { client: self }
    }

    pub fn account(&self) -> AccountApi<'_> {
        AccountApi {
            client: self,
            account_id: None,
        }
    }

    pub fn account_with_id<'a>(&'a self, account_id: impl Into<Cow<'a, str>>) -> AccountApi<'a> {
        AccountApi {
            client: self,
            account_id: Some(account_id.into()),
        }
    }

    pub fn instrument<'a>(&'a self, instrument: impl Into<Cow<'a, str>>) -> InstrumentApi<'a> {
        InstrumentApi {
            client: self,
            instrument: instrument.into(),
        }
    }

    pub fn orders(&self) -> OrdersApi<'_> {
        OrdersApi {
            client: self,
            account_id: None,
        }
    }

    pub fn orders_for_account<'a>(&'a self, account_id: impl Into<Cow<'a, str>>) -> OrdersApi<'a> {
        OrdersApi {
            client: self,
            account_id: Some(account_id.into()),
        }
    }

    pub fn trades(&self) -> TradesApi<'_> {
        TradesApi {
            client: self,
            account_id: None,
        }
    }

    pub fn trades_for_account<'a>(&'a self, account_id: impl Into<Cow<'a, str>>) -> TradesApi<'a> {
        TradesApi {
            client: self,
            account_id: Some(account_id.into()),
        }
    }

    pub fn positions(&self) -> PositionsApi<'_> {
        PositionsApi {
            client: self,
            account_id: None,
        }
    }

    pub fn positions_for_account<'a>(
        &'a self,
        account_id: impl Into<Cow<'a, str>>,
    ) -> PositionsApi<'a> {
        PositionsApi {
            client: self,
            account_id: Some(account_id.into()),
        }
    }

    pub fn pricing(&self) -> PricingApi<'_> {
        PricingApi {
            client: self,
            account_id: None,
        }
    }

    pub fn pricing_for_account<'a>(
        &'a self,
        account_id: impl Into<Cow<'a, str>>,
    ) -> PricingApi<'a> {
        PricingApi {
            client: self,
            account_id: Some(account_id.into()),
        }
    }

    pub fn transactions(&self) -> TransactionsApi<'_> {
        TransactionsApi {
            client: self,
            account_id: None,
        }
    }

    pub fn transactions_for_account<'a>(
        &'a self,
        account_id: impl Into<Cow<'a, str>>,
    ) -> TransactionsApi<'a> {
        TransactionsApi {
            client: self,
            account_id: Some(account_id.into()),
        }
    }
}

impl AccountsApi<'_> {
    pub async fn list(&self) -> OandaResult<Value> {
        self.client.get_json("/v3/accounts", &[]).await
    }
}

impl AccountApi<'_> {
    fn account_id(&self) -> OandaResult<&str> {
        match &self.account_id {
            Some(account_id) => Ok(account_id.as_ref()),
            None => self.client.require_account_id(),
        }
    }

    pub async fn get(&self) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .get_json(&format!("/v3/accounts/{id}"), &[])
            .await
    }

    pub async fn summary(&self) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .get_json(&format!("/v3/accounts/{id}/summary"), &[])
            .await
    }

    pub async fn instruments(&self) -> OandaResult<Value> {
        self.instruments_with(AccountInstrumentsParams::default())
            .await
    }

    pub async fn instruments_with(&self, params: AccountInstrumentsParams) -> OandaResult<Value> {
        let id = self.account_id()?;
        let mut query = Vec::new();
        push_opt(&mut query, "instruments", &params.instruments);
        self.client
            .get_json(&format!("/v3/accounts/{id}/instruments"), &query)
            .await
    }

    pub async fn configure(&self, body: Value) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .patch_json(&format!("/v3/accounts/{id}/configuration"), body)
            .await
    }

    pub async fn changes(&self, since_transaction_id: impl AsRef<str>) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .get_json(
                &format!("/v3/accounts/{id}/changes"),
                &[("sinceTransactionID", since_transaction_id.as_ref())],
            )
            .await
    }
}

impl InstrumentApi<'_> {
    pub async fn candles(&self) -> OandaResult<Value> {
        self.candles_with(CandlesParams::default()).await
    }

    pub async fn candles_with(&self, params: CandlesParams) -> OandaResult<Value> {
        let query = candles_query(&params);
        self.client
            .get_json(
                &format!("/v3/instruments/{}/candles", self.instrument),
                &query,
            )
            .await
    }
}

impl OrdersApi<'_> {
    fn account_id(&self) -> OandaResult<&str> {
        api_account_id(self.client, &self.account_id)
    }

    pub async fn create(&self, body: Value) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .post_json(&format!("/v3/accounts/{id}/orders"), body)
            .await
    }

    pub async fn list(&self) -> OandaResult<Value> {
        self.list_with(OrderListParams::default()).await
    }

    pub async fn list_with(&self, params: OrderListParams) -> OandaResult<Value> {
        let id = self.account_id()?;
        let mut query = Vec::new();
        push_opt(&mut query, "ids", &params.ids);
        push_opt(&mut query, "state", &params.state);
        push_opt(&mut query, "instrument", &params.instrument);
        push_opt(&mut query, "count", &params.count);
        push_opt(&mut query, "beforeID", &params.before_id);
        self.client
            .get_json(&format!("/v3/accounts/{id}/orders"), &query)
            .await
    }

    pub async fn pending(&self) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .get_json(&format!("/v3/accounts/{id}/pendingOrders"), &[])
            .await
    }

    pub async fn get(&self, order_specifier: impl AsRef<str>) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .get_json(
                &format!("/v3/accounts/{id}/orders/{}", order_specifier.as_ref()),
                &[],
            )
            .await
    }

    pub async fn replace(
        &self,
        order_specifier: impl AsRef<str>,
        body: Value,
    ) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .put_json(
                &format!("/v3/accounts/{id}/orders/{}", order_specifier.as_ref()),
                Some(body),
            )
            .await
    }

    pub async fn cancel(&self, order_specifier: impl AsRef<str>) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .put_json(
                &format!(
                    "/v3/accounts/{id}/orders/{}/cancel",
                    order_specifier.as_ref()
                ),
                None,
            )
            .await
    }

    pub async fn client_extensions(
        &self,
        order_specifier: impl AsRef<str>,
        body: Value,
    ) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .put_json(
                &format!(
                    "/v3/accounts/{id}/orders/{}/clientExtensions",
                    order_specifier.as_ref()
                ),
                Some(body),
            )
            .await
    }
}

impl TradesApi<'_> {
    fn account_id(&self) -> OandaResult<&str> {
        api_account_id(self.client, &self.account_id)
    }

    pub async fn list(&self) -> OandaResult<Value> {
        self.list_with(TradeListParams::default()).await
    }

    pub async fn list_with(&self, params: TradeListParams) -> OandaResult<Value> {
        let id = self.account_id()?;
        let mut query = Vec::new();
        push_opt(&mut query, "ids", &params.ids);
        push_opt(&mut query, "state", &params.state);
        push_opt(&mut query, "instrument", &params.instrument);
        push_opt(&mut query, "count", &params.count);
        push_opt(&mut query, "beforeID", &params.before_id);
        self.client
            .get_json(&format!("/v3/accounts/{id}/trades"), &query)
            .await
    }

    pub async fn open(&self) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .get_json(&format!("/v3/accounts/{id}/openTrades"), &[])
            .await
    }

    pub async fn get(&self, trade_specifier: impl AsRef<str>) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .get_json(
                &format!("/v3/accounts/{id}/trades/{}", trade_specifier.as_ref()),
                &[],
            )
            .await
    }

    pub async fn close(&self, trade_specifier: impl AsRef<str>) -> OandaResult<Value> {
        self.close_with_body(trade_specifier, None).await
    }

    pub async fn close_partial(
        &self,
        trade_specifier: impl AsRef<str>,
        body: Value,
    ) -> OandaResult<Value> {
        self.close_with_body(trade_specifier, Some(body)).await
    }

    pub async fn close_with_body(
        &self,
        trade_specifier: impl AsRef<str>,
        body: Option<Value>,
    ) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .put_json(
                &format!(
                    "/v3/accounts/{id}/trades/{}/close",
                    trade_specifier.as_ref()
                ),
                body,
            )
            .await
    }

    pub async fn client_extensions(
        &self,
        trade_specifier: impl AsRef<str>,
        body: Value,
    ) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .put_json(
                &format!(
                    "/v3/accounts/{id}/trades/{}/clientExtensions",
                    trade_specifier.as_ref()
                ),
                Some(body),
            )
            .await
    }

    pub async fn orders(
        &self,
        trade_specifier: impl AsRef<str>,
        body: Value,
    ) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .put_json(
                &format!(
                    "/v3/accounts/{id}/trades/{}/orders",
                    trade_specifier.as_ref()
                ),
                Some(body),
            )
            .await
    }
}

impl PositionsApi<'_> {
    fn account_id(&self) -> OandaResult<&str> {
        api_account_id(self.client, &self.account_id)
    }

    pub async fn list(&self) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .get_json(&format!("/v3/accounts/{id}/positions"), &[])
            .await
    }

    pub async fn open(&self) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .get_json(&format!("/v3/accounts/{id}/openPositions"), &[])
            .await
    }

    pub async fn get(&self, instrument: impl AsRef<str>) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .get_json(
                &format!("/v3/accounts/{id}/positions/{}", instrument.as_ref()),
                &[],
            )
            .await
    }

    pub async fn close(&self, instrument: impl AsRef<str>, body: Value) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .put_json(
                &format!("/v3/accounts/{id}/positions/{}/close", instrument.as_ref()),
                Some(body),
            )
            .await
    }
}

impl PricingApi<'_> {
    fn account_id(&self) -> OandaResult<&str> {
        api_account_id(self.client, &self.account_id)
    }

    pub async fn get(&self, instruments: impl Into<String>) -> OandaResult<Value> {
        self.get_with(instruments, PricingGetParams::default())
            .await
    }

    pub async fn get_with(
        &self,
        instruments: impl Into<String>,
        params: PricingGetParams,
    ) -> OandaResult<Value> {
        let id = self.account_id()?;
        let instruments = instruments.into();
        let mut query = vec![("instruments", instruments.as_str())];
        push_opt(&mut query, "since", &params.since);
        if params.include_units_available {
            query.push(("includeUnitsAvailable", "true"));
        }
        if params.include_home_conversions {
            query.push(("includeHomeConversions", "true"));
        }
        self.client
            .get_json(&format!("/v3/accounts/{id}/pricing"), &query)
            .await
    }

    pub async fn stream(&self, instruments: impl Into<String>) -> OandaResult<reqwest::Response> {
        self.stream_with(instruments, PricingStreamParams::default())
            .await
    }

    pub async fn stream_with(
        &self,
        instruments: impl Into<String>,
        params: PricingStreamParams,
    ) -> OandaResult<reqwest::Response> {
        let id = self.account_id()?;
        let instruments = instruments.into();
        let mut query = vec![("instruments", instruments.as_str())];
        if params.snapshot {
            query.push(("snapshot", "true"));
        }
        if params.include_home_conversions {
            query.push(("includeHomeConversions", "true"));
        }
        self.client
            .stream_response(&format!("/v3/accounts/{id}/pricing/stream"), &query)
            .await
    }

    pub async fn candles_latest(
        &self,
        candle_specifications: impl Into<String>,
    ) -> OandaResult<Value> {
        self.candles_latest_with(candle_specifications, PricingCandlesLatestParams::default())
            .await
    }

    pub async fn candles_latest_with(
        &self,
        candle_specifications: impl Into<String>,
        params: PricingCandlesLatestParams,
    ) -> OandaResult<Value> {
        let id = self.account_id()?;
        let candle_specifications = candle_specifications.into();
        let mut query = vec![("candleSpecifications", candle_specifications.as_str())];
        push_opt(&mut query, "units", &params.units);
        if params.smooth {
            query.push(("smooth", "true"));
        }
        push_opt(&mut query, "dailyAlignment", &params.daily_alignment);
        push_opt(&mut query, "alignmentTimezone", &params.alignment_timezone);
        push_opt(&mut query, "weeklyAlignment", &params.weekly_alignment);
        self.client
            .get_json(&format!("/v3/accounts/{id}/candles/latest"), &query)
            .await
    }

    pub async fn candles(&self, instrument: impl Into<String>) -> OandaResult<Value> {
        self.candles_with(instrument, PricingCandlesParams::default())
            .await
    }

    pub async fn candles_with(
        &self,
        instrument: impl Into<String>,
        params: PricingCandlesParams,
    ) -> OandaResult<Value> {
        let id = self.account_id()?;
        let instrument = instrument.into();
        let query = pricing_candles_query(&params);
        self.client
            .get_json(
                &format!("/v3/accounts/{id}/instruments/{instrument}/candles"),
                &query,
            )
            .await
    }
}

impl TransactionsApi<'_> {
    fn account_id(&self) -> OandaResult<&str> {
        api_account_id(self.client, &self.account_id)
    }

    pub async fn list(&self) -> OandaResult<Value> {
        self.list_with(TransactionListParams::default()).await
    }

    pub async fn list_with(&self, params: TransactionListParams) -> OandaResult<Value> {
        let id = self.account_id()?;
        let mut query = Vec::new();
        push_opt(&mut query, "from", &params.from);
        push_opt(&mut query, "to", &params.to);
        push_opt(&mut query, "pageSize", &params.page_size);
        push_opt(&mut query, "type", &params.type_filter);
        self.client
            .get_json(&format!("/v3/accounts/{id}/transactions"), &query)
            .await
    }

    pub async fn get(&self, transaction_id: impl AsRef<str>) -> OandaResult<Value> {
        let id = self.account_id()?;
        self.client
            .get_json(
                &format!("/v3/accounts/{id}/transactions/{}", transaction_id.as_ref()),
                &[],
            )
            .await
    }

    pub async fn id_range(
        &self,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> OandaResult<Value> {
        self.id_range_with_type(from, to, None::<String>).await
    }

    pub async fn id_range_with_type(
        &self,
        from: impl Into<String>,
        to: impl Into<String>,
        type_filter: Option<impl Into<String>>,
    ) -> OandaResult<Value> {
        let id = self.account_id()?;
        let from = from.into();
        let to = to.into();
        let type_filter = type_filter.map(Into::into);
        let mut query = vec![("from", from.as_str()), ("to", to.as_str())];
        push_opt(&mut query, "type", &type_filter);
        self.client
            .get_json(&format!("/v3/accounts/{id}/transactions/idrange"), &query)
            .await
    }

    pub async fn since_id(&self, id: impl Into<String>) -> OandaResult<Value> {
        self.since_id_with_type(id, None::<String>).await
    }

    pub async fn since_id_with_type(
        &self,
        since_id: impl Into<String>,
        type_filter: Option<impl Into<String>>,
    ) -> OandaResult<Value> {
        let id = self.account_id()?;
        let since_id = since_id.into();
        let type_filter = type_filter.map(Into::into);
        let mut query = vec![("id", since_id.as_str())];
        push_opt(&mut query, "type", &type_filter);
        self.client
            .get_json(&format!("/v3/accounts/{id}/transactions/sinceid"), &query)
            .await
    }

    pub async fn stream(&self) -> OandaResult<reqwest::Response> {
        let id = self.account_id()?;
        self.client
            .stream_response(&format!("/v3/accounts/{id}/transactions/stream"), &[])
            .await
    }
}

#[derive(Debug, Clone, Default)]
pub struct AccountInstrumentsParams {
    pub instruments: Option<String>,
}

impl AccountInstrumentsParams {
    pub fn instruments(mut self, instruments: impl Into<String>) -> Self {
        self.instruments = Some(instruments.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct CandlesParams {
    pub price: Option<String>,
    pub granularity: Option<String>,
    pub count: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub smooth: bool,
    pub include_first: Option<String>,
    pub daily_alignment: Option<String>,
    pub alignment_timezone: Option<String>,
    pub weekly_alignment: Option<String>,
}

impl CandlesParams {
    pub fn price(mut self, price: impl Into<String>) -> Self {
        self.price = Some(price.into());
        self
    }

    pub fn granularity(mut self, granularity: impl Into<String>) -> Self {
        self.granularity = Some(granularity.into());
        self
    }

    pub fn count(mut self, count: impl ToString) -> Self {
        self.count = Some(count.to_string());
        self
    }

    pub fn from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    pub fn to(mut self, to: impl Into<String>) -> Self {
        self.to = Some(to.into());
        self
    }

    pub fn smooth(mut self) -> Self {
        self.smooth = true;
        self
    }

    pub fn include_first(mut self, include_first: bool) -> Self {
        self.include_first = Some(include_first.to_string());
        self
    }

    pub fn daily_alignment(mut self, daily_alignment: impl ToString) -> Self {
        self.daily_alignment = Some(daily_alignment.to_string());
        self
    }

    pub fn alignment_timezone(mut self, alignment_timezone: impl Into<String>) -> Self {
        self.alignment_timezone = Some(alignment_timezone.into());
        self
    }

    pub fn weekly_alignment(mut self, weekly_alignment: impl Into<String>) -> Self {
        self.weekly_alignment = Some(weekly_alignment.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct OrderListParams {
    pub ids: Option<String>,
    pub state: Option<String>,
    pub instrument: Option<String>,
    pub count: Option<String>,
    pub before_id: Option<String>,
}

impl OrderListParams {
    pub fn ids(mut self, ids: impl Into<String>) -> Self {
        self.ids = Some(ids.into());
        self
    }

    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    pub fn instrument(mut self, instrument: impl Into<String>) -> Self {
        self.instrument = Some(instrument.into());
        self
    }

    pub fn count(mut self, count: impl ToString) -> Self {
        self.count = Some(count.to_string());
        self
    }

    pub fn before_id(mut self, before_id: impl Into<String>) -> Self {
        self.before_id = Some(before_id.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct TradeListParams {
    pub ids: Option<String>,
    pub state: Option<String>,
    pub instrument: Option<String>,
    pub count: Option<String>,
    pub before_id: Option<String>,
}

impl TradeListParams {
    pub fn ids(mut self, ids: impl Into<String>) -> Self {
        self.ids = Some(ids.into());
        self
    }

    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    pub fn instrument(mut self, instrument: impl Into<String>) -> Self {
        self.instrument = Some(instrument.into());
        self
    }

    pub fn count(mut self, count: impl ToString) -> Self {
        self.count = Some(count.to_string());
        self
    }

    pub fn before_id(mut self, before_id: impl Into<String>) -> Self {
        self.before_id = Some(before_id.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct PricingGetParams {
    pub since: Option<String>,
    pub include_units_available: bool,
    pub include_home_conversions: bool,
}

impl PricingGetParams {
    pub fn since(mut self, since: impl Into<String>) -> Self {
        self.since = Some(since.into());
        self
    }

    pub fn include_units_available(mut self) -> Self {
        self.include_units_available = true;
        self
    }

    pub fn include_home_conversions(mut self) -> Self {
        self.include_home_conversions = true;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct PricingStreamParams {
    pub snapshot: bool,
    pub include_home_conversions: bool,
}

impl PricingStreamParams {
    pub fn snapshot(mut self) -> Self {
        self.snapshot = true;
        self
    }

    pub fn include_home_conversions(mut self) -> Self {
        self.include_home_conversions = true;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct PricingCandlesLatestParams {
    pub units: Option<String>,
    pub smooth: bool,
    pub daily_alignment: Option<String>,
    pub alignment_timezone: Option<String>,
    pub weekly_alignment: Option<String>,
}

impl PricingCandlesLatestParams {
    pub fn units(mut self, units: impl ToString) -> Self {
        self.units = Some(units.to_string());
        self
    }

    pub fn smooth(mut self) -> Self {
        self.smooth = true;
        self
    }

    pub fn daily_alignment(mut self, daily_alignment: impl ToString) -> Self {
        self.daily_alignment = Some(daily_alignment.to_string());
        self
    }

    pub fn alignment_timezone(mut self, alignment_timezone: impl Into<String>) -> Self {
        self.alignment_timezone = Some(alignment_timezone.into());
        self
    }

    pub fn weekly_alignment(mut self, weekly_alignment: impl Into<String>) -> Self {
        self.weekly_alignment = Some(weekly_alignment.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct PricingCandlesParams {
    pub price: Option<String>,
    pub granularity: Option<String>,
    pub count: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub smooth: bool,
    pub include_first: Option<String>,
    pub daily_alignment: Option<String>,
    pub alignment_timezone: Option<String>,
    pub weekly_alignment: Option<String>,
    pub units: Option<String>,
}

impl PricingCandlesParams {
    pub fn price(mut self, price: impl Into<String>) -> Self {
        self.price = Some(price.into());
        self
    }

    pub fn granularity(mut self, granularity: impl Into<String>) -> Self {
        self.granularity = Some(granularity.into());
        self
    }

    pub fn count(mut self, count: impl ToString) -> Self {
        self.count = Some(count.to_string());
        self
    }

    pub fn from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    pub fn to(mut self, to: impl Into<String>) -> Self {
        self.to = Some(to.into());
        self
    }

    pub fn smooth(mut self) -> Self {
        self.smooth = true;
        self
    }

    pub fn include_first(mut self, include_first: bool) -> Self {
        self.include_first = Some(include_first.to_string());
        self
    }

    pub fn daily_alignment(mut self, daily_alignment: impl ToString) -> Self {
        self.daily_alignment = Some(daily_alignment.to_string());
        self
    }

    pub fn alignment_timezone(mut self, alignment_timezone: impl Into<String>) -> Self {
        self.alignment_timezone = Some(alignment_timezone.into());
        self
    }

    pub fn weekly_alignment(mut self, weekly_alignment: impl Into<String>) -> Self {
        self.weekly_alignment = Some(weekly_alignment.into());
        self
    }

    pub fn units(mut self, units: impl ToString) -> Self {
        self.units = Some(units.to_string());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct TransactionListParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub page_size: Option<String>,
    pub type_filter: Option<String>,
}

impl TransactionListParams {
    pub fn from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    pub fn to(mut self, to: impl Into<String>) -> Self {
        self.to = Some(to.into());
        self
    }

    pub fn page_size(mut self, page_size: impl ToString) -> Self {
        self.page_size = Some(page_size.to_string());
        self
    }

    pub fn type_filter(mut self, type_filter: impl Into<String>) -> Self {
        self.type_filter = Some(type_filter.into());
        self
    }
}

fn api_account_id<'a>(
    client: &'a OandaClient,
    account_id: &'a Option<Cow<'a, str>>,
) -> OandaResult<&'a str> {
    match account_id {
        Some(account_id) => Ok(account_id.as_ref()),
        None => client.require_account_id(),
    }
}

fn push_opt<'a>(query: &mut Vec<(&'a str, &'a str)>, key: &'a str, value: &'a Option<String>) {
    if let Some(value) = value {
        query.push((key, value.as_str()));
    }
}

fn candles_query(params: &CandlesParams) -> Vec<(&str, &str)> {
    let mut query = Vec::new();
    push_opt(&mut query, "price", &params.price);
    push_opt(&mut query, "granularity", &params.granularity);
    push_opt(&mut query, "count", &params.count);
    push_opt(&mut query, "from", &params.from);
    push_opt(&mut query, "to", &params.to);
    if params.smooth {
        query.push(("smooth", "true"));
    }
    push_opt(&mut query, "includeFirst", &params.include_first);
    push_opt(&mut query, "dailyAlignment", &params.daily_alignment);
    push_opt(&mut query, "alignmentTimezone", &params.alignment_timezone);
    push_opt(&mut query, "weeklyAlignment", &params.weekly_alignment);
    query
}

fn pricing_candles_query(params: &PricingCandlesParams) -> Vec<(&str, &str)> {
    let mut query = Vec::new();
    push_opt(&mut query, "price", &params.price);
    push_opt(&mut query, "granularity", &params.granularity);
    push_opt(&mut query, "count", &params.count);
    push_opt(&mut query, "from", &params.from);
    push_opt(&mut query, "to", &params.to);
    if params.smooth {
        query.push(("smooth", "true"));
    }
    push_opt(&mut query, "includeFirst", &params.include_first);
    push_opt(&mut query, "dailyAlignment", &params.daily_alignment);
    push_opt(&mut query, "alignmentTimezone", &params.alignment_timezone);
    push_opt(&mut query, "weeklyAlignment", &params.weekly_alignment);
    push_opt(&mut query, "units", &params.units);
    query
}
