#nullable enable

using System;
using System.Collections.Generic;
using System.Linq;
using UnityEngine;
using UnityEngine.Events;
using SpacetimeDB;
using SpacetimeDB.Types;

namespace SciFiEndlessRunner.API
{
    #region Wrapper Types

    [Serializable]
    public class EnergyData
    {
        public int CurrentEnergy;
        public int MaxEnergy;
        public EnergyChange? NextEnergyChangeAt;
        public TimeSpan RegenerationInterval;
        public TimeSpan ConsumptionInterval;

        public EnergyData(EnergyWidget widget)
        {
            CurrentEnergy = widget.CurrentEnergy;
            MaxEnergy = widget.MaxEnergy;
            NextEnergyChangeAt = widget.NextEnergyChangeAt;
            RegenerationInterval = TimeSpan.FromTicks(widget.RegenerationIntervalMicros * 10);
            ConsumptionInterval = TimeSpan.FromTicks(widget.ConsumptionIntervalMicros * 10);
        }
    }

    [Serializable]
    public class DailyRewardEntry
    {
        public int Day;
        public int Gems;

        public DailyRewardEntry(DailyRewardWidget widget)
        {
            Day = widget.Day;
            Gems = widget.Gems;
        }
    }

    [Serializable]
    public class DailyRewardData
    {
        public List<DailyRewardEntry> Rewards;
        public bool ShowEllipsisBeforeLastReward;
        public bool CanClaimFirstReward;

        public DailyRewardData(DailyRewards widget)
        {
            Rewards = widget.DailyRewardWidgetsShown.Select(w => new DailyRewardEntry(w)).ToList();
            ShowEllipsisBeforeLastReward = widget.ShowThreePointsBeforeLastDailyRewardWidget;
            CanClaimFirstReward = widget.CanClaimFirstDailyRewardWidget;
        }
    }

    [Serializable]
    public class MagnetData
    {
        public int Amount;
        public int RangeUpgradeLevel;
        public int DurationUpgradeLevel;
        public int SpawnChanceUpgradeLevel;
        public PricingMode PurchasePrice;
        public MagnetUpgradePrice RangeUpgradePrice;
        public MagnetUpgradePrice DurationUpgradePrice;
        public MagnetUpgradePrice SpawnChanceUpgradePrice;

        public MagnetData(MagnetWidgets widget)
        {
            Amount = widget.Amount;
            RangeUpgradeLevel = widget.RangeUpgradeLevel;
            DurationUpgradeLevel = widget.DurationUpgradeLevel;
            SpawnChanceUpgradeLevel = widget.SpawnChanceUpgradeLevel;
            PurchasePrice = widget.PurchasePrice;
            RangeUpgradePrice = widget.RangeUpgradePrice;
            DurationUpgradePrice = widget.DurationUpgradePrice;
            SpawnChanceUpgradePrice = widget.SpawnChanceUpgradePrice;
        }
    }

    [Serializable]
    public class ShieldData
    {
        public int Amount;
        public int CollisionsUpgradeLevel;
        public int DurationUpgradeLevel;
        public int SpawnChanceUpgradeLevel;
        public PricingMode PurchasePrice;
        public ShieldUpgradePrice CollisionsUpgradePrice;
        public ShieldUpgradePrice DurationUpgradePrice;
        public ShieldUpgradePrice SpawnChanceUpgradePrice;

        public ShieldData(ShieldWidgets widget)
        {
            Amount = widget.Amount;
            CollisionsUpgradeLevel = widget.CollisionsUpgradeLevel;
            DurationUpgradeLevel = widget.DurationUpgradeLevel;
            SpawnChanceUpgradeLevel = widget.SpawnChanceUpgradeLevel;
            PurchasePrice = widget.PurchasePrice;
            CollisionsUpgradePrice = widget.CollisionsUpgradePrice;
            DurationUpgradePrice = widget.DurationUpgradePrice;
            SpawnChanceUpgradePrice = widget.SpawnChanceUpgradePrice;
        }
    }

    [Serializable]
    public class ReviveData
    {
        public PricingMode Price;
        public bool AdAvailable;

        public ReviveData(RevivePanel widget)
        {
            Price = widget.Price;
            AdAvailable = widget.AdAvailable;
        }
    }

    [Serializable]
    public class EnergyPanelData
    {
        public PricingMode Price;
        public bool AdAvailable;

        public EnergyPanelData(EnergyPanel widget)
        {
            Price = widget.Price;
            AdAvailable = widget.AdAvailable;
        }
    }

    #endregion

    #region UnityEvent Subclasses

    [Serializable] public class EnergyDataEvent : UnityEvent<EnergyData> { }
    [Serializable] public class DailyRewardDataEvent : UnityEvent<DailyRewardData> { }
    [Serializable] public class MagnetDataEvent : UnityEvent<MagnetData> { }
    [Serializable] public class ShieldDataEvent : UnityEvent<ShieldData> { }
    [Serializable] public class ReviveDataEvent : UnityEvent<ReviveData> { }
    [Serializable] public class EnergyPanelDataEvent : UnityEvent<EnergyPanelData> { }
    [Serializable] public class UlongEvent : UnityEvent<ulong> { }
    [Serializable] public class UintEvent : UnityEvent<uint> { }
    [Serializable] public class StringEvent : UnityEvent<string> { }
    [Serializable] public class BoolStringEvent : UnityEvent<bool, string> { }
    [Serializable] public class PlayerSkinWidgetEntryListEvent : UnityEvent<List<PlayerSkinWidgetEntry>> { }
    [Serializable] public class LevelSkinWidgetEntryListEvent : UnityEvent<List<LevelSkinWidgetEntry>> { }
    [Serializable] public class PlayerMovementTrailWidgetEntryListEvent : UnityEvent<List<PlayerMovementTrailWidgetEntry>> { }

    #endregion

    public class GameAPI : MonoBehaviour
    {
        #region Singleton

        public static GameAPI Instance { get; private set; }

        [SerializeField] private string serverUrl = "http://localhost:3000";
        [SerializeField] private string moduleName = "sci-fi-endless-runner-server";

        #endregion

        #region Connection State

        private DbConnection connection;

        public bool IsConnected => connection != null && connection.IsActive;
        public Identity? LocalIdentity { get; private set; }

        #endregion

        #region Lifecycle Events

        public UnityEvent OnConnected = new();
        public UnityEvent OnDisconnected = new();
        public UnityEvent OnSubscriptionApplied = new();
        public StringEvent OnConnectionError = new();
        public StringEvent OnReducerError = new();
        public BoolStringEvent OnInAppPurchaseComplete = new();

        #endregion

        #region View Groups

        [Serializable]
        public class HudView
        {
            private readonly GameAPI api;

            public UlongEvent OnCoinsChanged = new();
            public UlongEvent OnGemsChanged = new();
            public UintEvent OnLevelChanged = new();
            public EnergyDataEvent OnEnergyChanged = new();
            public DailyRewardDataEvent OnDailyRewardsChanged = new();

            public ulong Coins =>
                api.connection?.Db.CoinWidget.Iter().FirstOrDefault()?.Amount ?? 0;

            public ulong Gems =>
                api.connection?.Db.GemWidget.Iter().FirstOrDefault()?.Amount ?? 0;

            public uint Level =>
                api.connection?.Db.LevelWidget.Iter().FirstOrDefault()?.Amount ?? 0;

            public EnergyData? Energy
            {
                get
                {
                    var widget = api.connection?.Db.EnergyWidget.Iter().FirstOrDefault();
                    return widget != null ? new EnergyData(widget) : null;
                }
            }

            public DailyRewardData? DailyRewards
            {
                get
                {
                    var widget = api.connection?.Db.DailyRewardWidgets.Iter().FirstOrDefault();
                    return widget != null ? new DailyRewardData(widget) : null;
                }
            }

            internal HudView(GameAPI api)
            {
                this.api = api;
            }

            internal void SubscribeToTableEvents(DbConnection conn)
            {
                conn.Db.CoinWidget.OnInsert += (ctx, row) =>
                    OnCoinsChanged.Invoke(row.Amount);
                conn.Db.CoinWidget.OnDelete += (ctx, row) =>
                    OnCoinsChanged.Invoke(Coins);

                conn.Db.GemWidget.OnInsert += (ctx, row) =>
                    OnGemsChanged.Invoke(row.Amount);
                conn.Db.GemWidget.OnDelete += (ctx, row) =>
                    OnGemsChanged.Invoke(Gems);

                conn.Db.LevelWidget.OnInsert += (ctx, row) =>
                    OnLevelChanged.Invoke(row.Amount);
                conn.Db.LevelWidget.OnDelete += (ctx, row) =>
                    OnLevelChanged.Invoke(Level);

                conn.Db.EnergyWidget.OnInsert += (ctx, row) =>
                    OnEnergyChanged.Invoke(new EnergyData(row));
                conn.Db.EnergyWidget.OnDelete += (ctx, row) =>
                {
                    var current = Energy;
                    if (current != null) OnEnergyChanged.Invoke(current);
                };

                conn.Db.DailyRewardWidgets.OnInsert += (ctx, row) =>
                    OnDailyRewardsChanged.Invoke(new DailyRewardData(row));
                conn.Db.DailyRewardWidgets.OnDelete += (ctx, row) =>
                {
                    var current = DailyRewards;
                    if (current != null) OnDailyRewardsChanged.Invoke(current);
                };
            }
        }

        [Serializable]
        public class ShopView
        {
            private readonly GameAPI api;

            public MagnetDataEvent OnMagnetChanged = new();
            public ShieldDataEvent OnShieldChanged = new();
            public PlayerSkinWidgetEntryListEvent OnPlayerSkinsChanged = new();
            public LevelSkinWidgetEntryListEvent OnLevelSkinsChanged = new();
            public PlayerMovementTrailWidgetEntryListEvent OnPlayerMovementTrailsChanged = new();

            public MagnetData? Magnet
            {
                get
                {
                    var widget = api.connection?.Db.MagnetWidgets.Iter().FirstOrDefault();
                    return widget != null ? new MagnetData(widget) : null;
                }
            }

            public ShieldData? Shield
            {
                get
                {
                    var widget = api.connection?.Db.ShieldWidgets.Iter().FirstOrDefault();
                    return widget != null ? new ShieldData(widget) : null;
                }
            }

            public List<PlayerSkinWidgetEntry> PlayerSkins =>
                api.connection?.Db.PlayerSkinWidgets.Iter().ToList() ?? new List<PlayerSkinWidgetEntry>();

            public List<LevelSkinWidgetEntry> LevelSkins =>
                api.connection?.Db.LevelSkinWidgets.Iter().ToList() ?? new List<LevelSkinWidgetEntry>();

            public List<PlayerMovementTrailWidgetEntry> PlayerMovementTrails =>
                api.connection?.Db.PlayerMovementTrailWidgets.Iter().ToList() ?? new List<PlayerMovementTrailWidgetEntry>();

            internal ShopView(GameAPI api)
            {
                this.api = api;
            }

            internal void SubscribeToTableEvents(DbConnection conn)
            {
                conn.Db.MagnetWidgets.OnInsert += (ctx, row) =>
                    OnMagnetChanged.Invoke(new MagnetData(row));
                conn.Db.MagnetWidgets.OnDelete += (ctx, row) =>
                {
                    var current = Magnet;
                    if (current != null) OnMagnetChanged.Invoke(current);
                };

                conn.Db.ShieldWidgets.OnInsert += (ctx, row) =>
                    OnShieldChanged.Invoke(new ShieldData(row));
                conn.Db.ShieldWidgets.OnDelete += (ctx, row) =>
                {
                    var current = Shield;
                    if (current != null) OnShieldChanged.Invoke(current);
                };

                conn.Db.PlayerSkinWidgets.OnInsert += (ctx, row) =>
                    OnPlayerSkinsChanged.Invoke(PlayerSkins);
                conn.Db.PlayerSkinWidgets.OnDelete += (ctx, row) =>
                    OnPlayerSkinsChanged.Invoke(PlayerSkins);

                conn.Db.LevelSkinWidgets.OnInsert += (ctx, row) =>
                    OnLevelSkinsChanged.Invoke(LevelSkins);
                conn.Db.LevelSkinWidgets.OnDelete += (ctx, row) =>
                    OnLevelSkinsChanged.Invoke(LevelSkins);

                conn.Db.PlayerMovementTrailWidgets.OnInsert += (ctx, row) =>
                    OnPlayerMovementTrailsChanged.Invoke(PlayerMovementTrails);
                conn.Db.PlayerMovementTrailWidgets.OnDelete += (ctx, row) =>
                    OnPlayerMovementTrailsChanged.Invoke(PlayerMovementTrails);
            }
        }

        [Serializable]
        public class GameplayView
        {
            private readonly GameAPI api;

            public EnergyPanelDataEvent OnEnergyPanelChanged = new();
            public ReviveDataEvent OnRevivePanelChanged = new();

            public EnergyPanelData? EnergyPanel
            {
                get
                {
                    var widget = api.connection?.Db.EnergyPanel.Iter().FirstOrDefault();
                    return widget != null ? new EnergyPanelData(widget) : null;
                }
            }

            public ReviveData? RevivePanel
            {
                get
                {
                    var widget = api.connection?.Db.RevivePanel.Iter().FirstOrDefault();
                    return widget != null ? new ReviveData(widget) : null;
                }
            }

            internal GameplayView(GameAPI api)
            {
                this.api = api;
            }

            internal void SubscribeToTableEvents(DbConnection conn)
            {
                conn.Db.EnergyPanel.OnInsert += (ctx, row) =>
                    OnEnergyPanelChanged.Invoke(new EnergyPanelData(row));
                conn.Db.EnergyPanel.OnDelete += (ctx, row) =>
                {
                    var current = EnergyPanel;
                    if (current != null) OnEnergyPanelChanged.Invoke(current);
                };

                conn.Db.RevivePanel.OnInsert += (ctx, row) =>
                    OnRevivePanelChanged.Invoke(new ReviveData(row));
                conn.Db.RevivePanel.OnDelete += (ctx, row) =>
                {
                    var current = RevivePanel;
                    if (current != null) OnRevivePanelChanged.Invoke(current);
                };
            }
        }

        public HudView Hud { get; private set; }
        public ShopView Shop { get; private set; }
        public GameplayView Gameplay { get; private set; }

        #endregion

        #region Reducers — Playthrough

        public void BeginPlaythrough()
        {
            connection.Reducers.BeginPlaythrough();
        }

        public void EndPlaythrough(EndReason endReason, ulong score, ulong coins, ulong gems, bool isHighScore, uint levels)
        {
            connection.Reducers.EndPlaythrough(endReason, score, coins, gems, isHighScore, levels);
        }

        public void PausePlaythrough(PauseReason pauseReason)
        {
            connection.Reducers.PausePlaythrough(pauseReason);
        }

        public void ContinuePlaythrough()
        {
            connection.Reducers.ContinuePlaythrough();
        }

        #endregion

        #region Reducers — Power-Ups

        public void UseShield()
        {
            connection.Reducers.UsePowerUp(PowerUpType.Shield);
        }

        public void UseMagnet()
        {
            connection.Reducers.UsePowerUp(PowerUpType.Magnet);
        }

        #endregion

        #region Reducers — Cosmetics

        public void ApplyPlayerSkin(PlayerSkinVariant variant)
        {
            connection.Reducers.ApplyCosmetic(new CosmeticVariant.PlayerSkin(variant));
        }

        public void ApplyLevelSkin(LevelSkinVariant variant)
        {
            connection.Reducers.ApplyCosmetic(new CosmeticVariant.LevelSkin(variant));
        }

        public void ApplyPlayerMovementTrail(PlayerMovementTrailVariant variant)
        {
            connection.Reducers.ApplyCosmetic(new CosmeticVariant.PlayerMovementTrail(variant));
        }

        #endregion

        #region Reducers — Purchases

        public void PurchaseEnergy(Currency? currency = null)
        {
            connection.Reducers.MakePurchase(new PurchaseVariant.Energy(default), currency);
        }

        public void PurchaseRevive(Currency? currency = null)
        {
            connection.Reducers.MakePurchase(new PurchaseVariant.Revive(default), currency);
        }

        public void PurchaseMagnet(Currency? currency = null)
        {
            connection.Reducers.MakePurchase(new PurchaseVariant.Magnet(default), currency);
        }

        public void PurchaseMagnetRangeUpgrade(Currency? currency = null)
        {
            connection.Reducers.MakePurchase(new PurchaseVariant.MagnetRangeUpgrade(default), currency);
        }

        public void PurchaseMagnetDurationUpgrade(Currency? currency = null)
        {
            connection.Reducers.MakePurchase(new PurchaseVariant.MagnetDurationUpgrade(default), currency);
        }

        public void PurchaseMagnetSpawnChanceUpgrade(Currency? currency = null)
        {
            connection.Reducers.MakePurchase(new PurchaseVariant.MagnetSpawnChanceUpgrade(default), currency);
        }

        public void PurchaseShield(Currency? currency = null)
        {
            connection.Reducers.MakePurchase(new PurchaseVariant.Shield(default), currency);
        }

        public void PurchaseShieldCollisionsUpgrade(Currency? currency = null)
        {
            connection.Reducers.MakePurchase(new PurchaseVariant.ShieldCollisionsUpgrade(default), currency);
        }

        public void PurchaseShieldDurationUpgrade(Currency? currency = null)
        {
            connection.Reducers.MakePurchase(new PurchaseVariant.ShieldDurationUpgrade(default), currency);
        }

        public void PurchaseShieldSpawnChanceUpgrade(Currency? currency = null)
        {
            connection.Reducers.MakePurchase(new PurchaseVariant.ShieldSpawnChanceUpgrade(default), currency);
        }

        public void PurchasePlayerSkin(PlayerSkinVariant variant, Currency? currency = null)
        {
            connection.Reducers.MakePurchase(new PurchaseVariant.PlayerSkin(variant), currency);
        }

        public void PurchaseLevelSkin(LevelSkinVariant variant, Currency? currency = null)
        {
            connection.Reducers.MakePurchase(new PurchaseVariant.LevelSkin(variant), currency);
        }

        public void PurchasePlayerMovementTrail(PlayerMovementTrailVariant variant, Currency? currency = null)
        {
            connection.Reducers.MakePurchase(new PurchaseVariant.PlayerMovementTrail(variant), currency);
        }

        #endregion

        #region Reducers — Daily Rewards

        public void ClaimDailyReward()
        {
            connection.Reducers.ClaimDailyReward();
        }

        #endregion

        #region Reducers — Player

        public void RenamePlayer(string name)
        {
            connection.Reducers.RenamePlayer(name);
        }

        public void SyncTime(short timeDifferenceFromUtcInMinutes)
        {
            connection.Reducers.SyncTime(timeDifferenceFromUtcInMinutes);
        }

        #endregion

        #region Reducers — Ads

        public void BeginReviveAdWatch()
        {
            connection.Reducers.BeginAdWatch(AdType.Revive);
        }

        public void BeginDoubleCoinsAdWatch()
        {
            connection.Reducers.BeginAdWatch(AdType.DoubleCoins);
        }

        public void BeginGemsAdWatch()
        {
            connection.Reducers.BeginAdWatch(AdType.Gems);
        }

        public void BeginEnergyAdWatch()
        {
            connection.Reducers.BeginAdWatch(AdType.Energy);
        }

        public void EndAdWatch(ulong adWatchId, AdWatchStatus status)
        {
            connection.Reducers.EndAdWatch(adWatchId, status);
        }

        #endregion

        #region Procedures — In-App Purchase

        public void HandleInAppPurchase(string purchaseToken)
        {
            connection.Procedures.HandleInAppPurchase(purchaseToken, (ctx, result) =>
            {
                if (result.IsSuccess)
                {
                    OnInAppPurchaseComplete.Invoke(true, string.Empty);
                }
                else
                {
                    OnInAppPurchaseComplete.Invoke(false, result.Error?.ToString() ?? "Unknown error");
                }
            });
        }

        #endregion

        #region Unity Lifecycle

        private void Awake()
        {
            if (Instance != null && Instance != this)
            {
                Destroy(gameObject);
                return;
            }

            Instance = this;
            DontDestroyOnLoad(gameObject);
        }

        private void Start()
        {
            Hud = new HudView(this);
            Shop = new ShopView(this);
            Gameplay = new GameplayView(this);

            var builder = DbConnection.Builder()
                .WithUri(serverUrl)
                .WithModuleName(moduleName)
                .OnConnect(HandleConnect)
                .OnConnectError(HandleConnectError)
                .OnDisconnect(HandleDisconnect);

            if (!string.IsNullOrEmpty(AuthToken.Token))
            {
                builder = builder.WithToken(AuthToken.Token);
            }

            connection = builder.Build();
        }

        private void Update()
        {
            connection?.FrameTick();
        }

        private void OnDestroy()
        {
            if (Instance != this) return;

            if (connection != null && connection.IsActive)
            {
                connection.Disconnect();
            }

            connection = null;
            Instance = null;
        }

        #endregion

        #region Connection Callbacks

        private void HandleConnect(DbConnection conn, Identity identity, string authToken)
        {
            AuthToken.SaveToken(authToken);
            LocalIdentity = identity;

            conn.OnUnhandledReducerError += (ctx, exception) =>
                OnReducerError.Invoke(exception.Message);

            Hud.SubscribeToTableEvents(conn);
            Shop.SubscribeToTableEvents(conn);
            Gameplay.SubscribeToTableEvents(conn);

            conn.SubscriptionBuilder()
                .OnApplied(HandleSubscriptionApplied)
                .SubscribeToAllTables();

            OnConnected.Invoke();
        }

        private void HandleConnectError(Exception exception)
        {
            Debug.LogError($"SpacetimeDB connection error: {exception}");
            OnConnectionError.Invoke(exception.Message);
        }

        private void HandleDisconnect(DbConnection conn, Exception? exception)
        {
            if (exception != null)
            {
                Debug.LogError($"SpacetimeDB disconnected with error: {exception}");
            }

            OnDisconnected.Invoke();
        }

        private void HandleSubscriptionApplied(SubscriptionEventContext ctx)
        {
            OnSubscriptionApplied.Invoke();
        }

        #endregion
    }
}
